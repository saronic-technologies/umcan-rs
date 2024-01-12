use anyhow::Context;
use socketcan::{BlockingCan, CanFrame, CanSocket, EmbeddedFrame, Id, Socket};
use std::env;
use umcan::{Message, MotorCmd, Telemetry};

fn main() -> anyhow::Result<()> {
    const CMD_ID: u32 = 0x03;
    const TELEM_ID: u32 = 0x7F;

    let iface = env::args().nth(1).unwrap_or_else(|| "vcan0".into());

    let mut read_sock = CanSocket::open(&iface)
        .with_context(|| format!("Failed to open socket on interface {}", iface))?;

    let mut write_sock = CanSocket::open(&iface)
        .with_context(|| format!("Failed to open socket on interface {}", iface))?;

    std::thread::spawn(move || loop {
        let frame = read_sock.receive().context("Receiving Frame").unwrap();
        match frame.id() {
            Id::Standard(_) => println!("Unsupported Standard ID"),
            Id::Extended(eid) => match eid.as_raw() {
                CMD_ID => println!("CMD: {:?}", MotorCmd::from(frame)),
                TELEM_ID => println!("TELEMETRY: {:?}", Telemetry::from(frame)),
                _ => println!("Unsupported CAN ID: {}", eid.as_raw()),
            },
        };
    });

    let sleep_time = std::time::Duration::from_millis(500);

    loop {
        let cmd = MotorCmd::new(0x8000);
        let frame = Message::MotorCmd(cmd).framify::<CanFrame>(CMD_ID).unwrap();
        write_sock.transmit(&frame).context("Transmitting frame")?;

        std::thread::sleep(sleep_time);

        let telem = Telemetry::new(0xdeadbeef, 0x8000, 128, -25);
        let tfd = Message::Telemetry(telem)
            .framify::<CanFrame>(TELEM_ID)
            .unwrap();
        write_sock.write_frame(&tfd).context("Transmitting frame")?;

        std::thread::sleep(sleep_time);
    }
}
