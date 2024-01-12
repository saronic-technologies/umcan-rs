#![no_std]
use embedded_can::{ExtendedId, Frame};

use binrw::io::Cursor;
use binrw::{binrw, BinRead, BinWrite};

#[binrw]
#[brw(little)]
#[derive(Debug)]
pub struct Telemetry {
    #[br(map(|x: (u8, u8, u8)| x.0 as u32 | ((x.1 as u32) << 8) | ((x.2 as u32) << 16)))]
    #[bw(map(|x: &u32| [*x as u8, (*x >> 8) as u8, (*x >> 16) as u8]))]
    pub status: u32,
    pub position: u16,
    pub current: u16,
    #[br(map(|x: u8| ((x as i16) - 50)))]
    #[bw(map(|x: &i16| (*x + 50) as u8))]
    pub temp: i16,
}

#[binrw]
#[brw(little)]
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
    pub fn new(status: u32, position: u16, current: u16, temp: i16) -> Self {
        Self {
            status,
            position,
            current,
            temp,
        }
    }
}

impl Message {
    pub fn framify<T: Frame>(&self, can_id: u32) -> Option<T> {
        match self {
            Self::MotorCmd(m) => {
                let id = ExtendedId::new(can_id).unwrap();
                T::new(id, &m.cmd_value.to_le_bytes())
            }
            Self::Telemetry(t) => {
                let id = ExtendedId::new(can_id).unwrap();
                let mut b = Cursor::new([0u8; 8]);
                let _ = t.write_le(&mut b);
                let bytes = b.into_inner();
                T::new(id, &bytes)
            }
            Self::Unsupported => return None,
        }
    }
}

impl<T: Frame> From<T> for MotorCmd {
    fn from(frame: T) -> Self {
        let data: &[u8] = frame.data();
        let mut bytes = Cursor::new(data);
        MotorCmd::read_le(&mut bytes).unwrap()
    }
}

impl<T: Frame> From<T> for Telemetry {
    fn from(frame: T) -> Self {
        let data: &[u8] = frame.data();
        let mut bytes = Cursor::new(data);
        Telemetry::read_le(&mut bytes).unwrap()
    }
}
