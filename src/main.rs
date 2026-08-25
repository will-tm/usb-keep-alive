#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU8, Ordering};

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
#[cfg(feature = "rp2040")]
use embassy_rp::flash::{Blocking, Flash};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_time::Timer;
use embassy_usb::class::hid::{HidBootProtocol, HidSubclass, HidWriter, State};
use embassy_usb::Handler;
use static_cell::StaticCell;
use usbd_hid::descriptor::{MouseReport, SerializedDescriptor};

use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
});

const DEVICE_STATE_NOT_MOUNTED: u8 = 0;
const DEVICE_STATE_MOUNTED: u8 = 1;
const DEVICE_STATE_SUSPENDED: u8 = 2;

static DEVICE_STATE: AtomicU8 = AtomicU8::new(DEVICE_STATE_NOT_MOUNTED);

struct DeviceHandler;

impl Handler for DeviceHandler {
    fn configured(&mut self, configured: bool) {
        if configured {
            info!("USB configured (mounted)");
            DEVICE_STATE.store(DEVICE_STATE_MOUNTED, Ordering::Relaxed);
        } else {
            info!("USB deconfigured (not mounted)");
            DEVICE_STATE.store(DEVICE_STATE_NOT_MOUNTED, Ordering::Relaxed);
        }
    }

    fn suspended(&mut self, suspended: bool) {
        if suspended {
            info!("USB suspended");
            DEVICE_STATE.store(DEVICE_STATE_SUSPENDED, Ordering::Relaxed);
        } else {
            info!("USB resumed");
            DEVICE_STATE.store(DEVICE_STATE_MOUNTED, Ordering::Relaxed);
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Unique 8-byte ID for the serial number: SPI flash UID on RP2040,
    // OTP chip ID on RP2350 (which has no flash UID accessor).
    #[cfg(feature = "rp2040")]
    let uid = {
        let mut flash = Flash::<_, Blocking, { 2 * 1024 * 1024 }>::new_blocking(p.FLASH);
        let mut uid = [0u8; 8];
        flash.blocking_unique_id(&mut uid).unwrap();
        uid
    };
    #[cfg(feature = "rp2350")]
    let uid = embassy_rp::otp::get_chipid().unwrap().to_be_bytes();

    static SERIAL_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let serial_buf = SERIAL_BUF.init([0u8; 16]);
    // Format as hex string
    for (i, byte) in uid.iter().enumerate() {
        serial_buf[i * 2] = hex_nibble(byte >> 4);
        serial_buf[i * 2 + 1] = hex_nibble(byte & 0x0F);
    }
    let serial_str = core::str::from_utf8(serial_buf).unwrap();
    info!("Serial: {}", serial_str);

    // USB driver
    let driver = Driver::new(p.USB, Irqs);

    // USB device config
    let mut config = embassy_usb::Config::new(0x1506, 0x4004);
    config.manufacturer = Some("will_tm");
    config.product = Some("USB Keep Alive");
    config.serial_number = Some(serial_str);
    config.max_power = 100;
    config.supports_remote_wakeup = true;

    // Allocate USB buffers
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static MSOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    let config_desc = CONFIG_DESC.init([0; 256]);
    let bos_desc = BOS_DESC.init([0; 256]);
    let msos_desc = MSOS_DESC.init([0; 256]);
    let control_buf = CONTROL_BUF.init([0; 64]);

    static HANDLER: StaticCell<DeviceHandler> = StaticCell::new();
    let handler = HANDLER.init(DeviceHandler);

    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        config_desc,
        bos_desc,
        msos_desc,
        control_buf,
    );

    builder.handler(handler);

    // HID mouse
    static HID_STATE: StaticCell<State> = StaticCell::new();
    let hid_state = HID_STATE.init(State::new());

    let hid_config = embassy_usb::class::hid::Config {
        report_descriptor: MouseReport::desc(),
        request_handler: None,
        poll_ms: 5,
        max_packet_size: 8,
        hid_subclass: HidSubclass::Boot,
        hid_boot_protocol: HidBootProtocol::Mouse,
    };

    let hid_writer = HidWriter::<_, 8>::new(&mut builder, hid_state, hid_config);

    // Build USB device
    let mut usb = builder.build();

    // Run all three tasks concurrently
    let usb_fut = usb.run();
    let mouse_fut = mouse_task(hid_writer);

    join(usb_fut, mouse_fut).await;
}

async fn mouse_task<'d>(mut writer: HidWriter<'d, Driver<'d, USB>, 8>) {
    loop {
        Timer::after_secs(10).await;

        let state = DEVICE_STATE.load(Ordering::Relaxed);
        if state != DEVICE_STATE_MOUNTED {
            continue;
        }

        // Move +1 pixel
        let report = MouseReport {
            buttons: 0,
            x: 1,
            y: 0,
            wheel: 0,
            pan: 0,
        };
        if let Err(_e) = writer.write_serialize(&report).await {
            warn!("Mouse write error");
            continue;
        }

        Timer::after_millis(20).await;

        // Move -1 pixel
        let report = MouseReport {
            buttons: 0,
            x: -1,
            y: 0,
            wheel: 0,
            pan: 0,
        };
        if let Err(_e) = writer.write_serialize(&report).await {
            warn!("Mouse write error");
        }
    }
}

fn hex_nibble(n: u8) -> u8 {
    match n {
        0..=9 => b'0' + n,
        _ => b'a' + (n - 10),
    }
}
