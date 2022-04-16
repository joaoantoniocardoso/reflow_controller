#![no_std]
#![no_main]

use lib as _; // global logger + panicking-behavior + memory layout

mod protocol;
use protocol::Protocol;

use defmt::{debug, error, info, panic, trace, warn};
use heapless::Vec;

use cortex_m::asm::delay;
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use stm32f1xx_hal::{pac, prelude::*};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("info ON");
    trace!("trace ON");
    warn!("warn ON");
    debug!("debug ON");
    error!("error ON");

    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(72.MHz())
        .hclk(72.MHz())
        .pclk1(36.MHz())
        .pclk2(72.MHz())
        .adcclk(12.MHz())
        .freeze(&mut flash.acr);

    if !clocks.usbclk_valid() {
        panic!("Can't configure USB Clock.");
    }
    info!("USB Ok!");

    // LED

    // Configure the on-board LED (PC13, green)
    let mut gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    led.set_high(); // Turn off

    let mut gpioa = dp.GPIOA.split();

    // USB

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low();
    delay(clocks.sysclk().raw() / 100);

    let usb = Peripheral {
        usb: dp.USB,
        pin_dm: gpioa.pa11,
        pin_dp: usb_dp.into_floating_input(&mut gpioa.crh),
    };
    let usb_bus = UsbBus::new(usb);

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("ETA CPS LABS")
        .product("Reflow Controller")
        .serial_number("DEV")
        .device_class(USB_CLASS_CDC)
        .build();

    let mut ready = false;
    let mut recv_buf = Vec::<u8, 1024>::new();
    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        // This 64 bytes for buffer size is defined elsewhere in the library,
        // it seems to be the maximum for the bluepill to be able to use USB in its full speed.
        // It is possible to change it, but requires some effort defining our own buffers,
        // which doesn't sound appealing for now.
        // Because our messages are bigger than 64 bytes, we are going to be concatenating the
        // received chunks until the end of the message, defined by a "\r\n".
        let mut chunk_buf = [0u8; 64];
        match serial.read(&mut chunk_buf) {
            Ok(chunk_buf_count) if chunk_buf_count > 0 => {
                led.set_high();
                // debug!("Chunk received: {}", chunk_buf[0..chunk_buf_count]);

                if recv_buf.capacity() < chunk_buf_count {
                    recv_buf.clear();
                    // debug!("Recv Buffer Cleared.");
                }
                recv_buf
                    .extend_from_slice(&chunk_buf[0..chunk_buf_count])
                    .unwrap();

                if recv_buf[(recv_buf.len() - 2)..recv_buf.len()] == *"\r\n".as_bytes() {
                    ready = true;
                    recv_buf.truncate(recv_buf.len() - 2);
                    // debug!("Recv buffer got ready!");
                }

                // debug!("recv buffer: {}", recv_buf[0..recv_buf.len()]);
            }
            _ => {}
        }

        if ready {
            let message = match serde_json_core::from_slice::<Protocol>(&recv_buf) {
                Ok(protocol) => Some(protocol.0),
                Err(_) => {
                    error!("Failed descerializing buffer: {:#?}", &recv_buf.as_slice());
                    None
                }
            };
            if let Some(message) = message {
                match message {
                    Protocol::Profile(profile) => {
                        debug!("Profile message received: {:#?}", &profile);
                    }
                    Protocol::Feedback(feedback) => {
                        debug!("Feedback message received: {:#?}", &feedback);
                    }
                    Protocol::Start(start) => {
                        debug!("Start message received: {}", &start);
                    }
                    Protocol::Stop(stop) => {
                        debug!("Stop message received: {:#?}", &stop);
                    }
                }
            };
            ready = false;
            recv_buf.clear();
        }
        led.set_low();
    }
}
