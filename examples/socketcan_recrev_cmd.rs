use anyhow::Context;
use embedded_can::blocking::Can;
use umcan::{Message, MotorCmd, Telemetry};
use socketcan::{CanFrame, CanSocket, Socket};
use std::env;

fn main() -> anyhow::Result<()> {
    let iface = env::args().nth(1).unwrap_or_else(|| "vcan0".into());

    let mut read_sock = CanSocket::open(&iface)
        .with_context(|| format!("Failed to open socket on interface {}", iface))?;

    let mut write_sock = CanSocket::open(&iface)
        .with_context(|| format!("Failed to open socket on interface {}", iface))?;

    std::thread::spawn(move || {
        loop {
            let frame = read_sock.receive().context("Receiving Frame").unwrap();
            let msg = Message::from(frame);
            match msg {
                Message::MotorCmd(m) => println!("{:?}", m),
                Message::Telemetry(t) => println!("{:?}", t),
                Message::Unsupported => println!("Unsupported CAN Frame"),
            }
        }
    });

    let sleep_time = std::time::Duration::from_millis(500);

    loop {
        let cmd = MotorCmd::new(0x8000);
        let frame = Message::MotorCmd(cmd).framify::<CanFrame>().unwrap();
        write_sock.transmit(&frame).context("Transmitting frame")?;

        std::thread::sleep(sleep_time);

        let telem = Telemetry::new(0xdeadbeef, 0x8000, 128, -25);
        let tfd = Message::Telemetry(telem).framify::<CanFrame>().unwrap();
        write_sock.write_frame(&tfd).context("Transmitting frame")?;

        std::thread::sleep(sleep_time);
    }
}
