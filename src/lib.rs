#![no_std]
use embedded_can::{ExtendedId, Frame, Id};

#[derive(Debug)]
pub struct Telemetry {
    pub status: u32,
    pub position: u16,
    pub current: u16,
    pub temp: i16,
}

#[derive(Debug)]
pub struct MotorCmd {
    pub cmd_value: u16,
}

pub enum Message {
    Telemetry(Telemetry),
    MotorCmd(MotorCmd),
    Unsupported,
}

impl MotorCmd {
    pub fn new(cmd_value: u16) -> Self {
        Self { cmd_value }
    }
}

impl Telemetry {
    pub fn new(
        status: u32,
        position: u16,
        current: u16,
        temp: i16,
    ) -> Self {
        Self {
            status,
            position,
            current,
            temp,
        }
    }
}

impl Message {
    pub fn framify<T: Frame>(&self) -> Option<T> {
        match self {
            Self::Telemetry(t) => {
                let id = ExtendedId::new(0x7f).unwrap();
                let mut b = [0u8; 8];
                b[0..3].copy_from_slice(&(&t.status.to_le_bytes())[0..3]);
                b[3..5].copy_from_slice(&t.position.to_le_bytes());
                b[5..7].copy_from_slice(&t.current.to_le_bytes());
                b[7..].copy_from_slice(&(&(t.temp + 50).to_le_bytes())[0..1]);
                T::new(id, &b)
            }
            Self::MotorCmd(m) => {
                let id = ExtendedId::new(0x03).unwrap();
                T::new(id, &m.cmd_value.to_le_bytes())
            }
            Self::Unsupported => return None,
        }
    }
}

impl<T: Frame> From<T> for Message {
    fn from(frame: T) -> Self {
        // Frame should be a CAN-FD frame
        let id = match frame.id() {
            Id::Standard(_) => return Self::Unsupported,
            Id::Extended(eid) => eid.as_raw(),
        };

        match id {
            // ctrl_id
            0x03 => {
                let data: &[u8] = frame.data();
                Self::MotorCmd(MotorCmd {
                    cmd_value: u16::from_le_bytes([data[0], data[1]]),
                })
            }
            //telem_id
            0x7f => {
                let data: &[u8] = frame.data();
                let mut status: u32 = 0;
                status |= data[0] as u32 | (data[1] as u32) << 8 | (data[2] as u32) << 16;
                Self::Telemetry(Telemetry {
                    status,
                    position: u16::from_le_bytes(data[3..5].try_into().unwrap()),
                    current: u16::from_le_bytes(data[5..7].try_into().unwrap()),
                    temp: (data[7] as i16) - 50,
                })
            }
            _ => Self::Unsupported,
        }
    }
}
