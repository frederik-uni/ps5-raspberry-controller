use std::fmt;

use bitflags::bitflags;

use super::internal::{Axis2D, Axis3D, Buttons, ControllerStateInternal};

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct FaceButtons: u8 {
        const SQUARE   = 0b0001_0000;
        const CROSS    = 0b0010_0000;
        const CIRCLE   = 0b0100_0000;
        const TRIANGLE = 0b1000_0000;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct ShoulderButtons: u8 {
        const L1      = 0b0000_0001;
        const R1      = 0b0000_0010;
        const L2      = 0b0000_0100;
        const R2      = 0b0000_1000;
        const CREATE  = 0b0001_0000;
        const OPTIONS = 0b0010_0000;
        const L3      = 0b0100_0000;
        const R3      = 0b1000_0000;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct SystemButtons: u8 {
        const PS        = 0b0000_0001;
        const TOUCHPAD  = 0b0000_0010;
        const MUTE      = 0b0000_0100;
        // Remaining 5 bits are vendor-defined
    }
}

// Optional: Hat directions as constants
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum HatDirection {
    Neutral = 0x08,
    North = 0x00,
    NorthEast = 0x01,
    East = 0x02,
    SouthEast = 0x03,
    South = 0x04,
    SouthWest = 0x05,
    West = 0x06,
    NorthWest = 0x07,
}

pub struct ParsedInput {
    pub report_id: u8,
    pub lx: u8,
    pub ly: u8,
    pub rx: u8,
    pub ry: u8,
    pub l2_axis: u8,
    pub r2_axis: u8,
    pub hat: HatDirection,
    pub face_buttons: FaceButtons,
    pub shoulder_buttons: ShoulderButtons,
    pub system_buttons: SystemButtons,
    pub battery_level: u8,
    pub ts: u32,
    pub gx: i16,
    pub gy: i16,
    pub gz: i16,
    pub ax: i16,
    pub ay: i16,
    pub az: i16,
}

impl fmt::Debug for ParsedInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParsedInput")
            .field("report_id", &self.report_id)
            .field("lx", &self.lx)
            .field("ly", &self.ly)
            .field("rx", &self.rx)
            .field("ry", &self.ry)
            .field("l2_axis", &self.l2_axis)
            .field("r2_axis", &self.r2_axis)
            .field("hat", &self.hat)
            .field("face_buttons", &self.face_buttons)
            .field("shoulder_buttons", &self.shoulder_buttons)
            .field("system_buttons", &self.system_buttons)
            // vendor_0x20, vendor_0x21, vendor_0x22 intentionally omitted
            .finish()
    }
}

impl ParsedInput {
    pub fn from_ps4_buf(buf: &[u8; 64]) -> Self {
        let hat = match buf[5] & 0x0F {
            0x00 => HatDirection::North,
            0x01 => HatDirection::NorthEast,
            0x02 => HatDirection::East,
            0x03 => HatDirection::SouthEast,
            0x04 => HatDirection::South,
            0x05 => HatDirection::SouthWest,
            0x06 => HatDirection::West,
            0x07 => HatDirection::NorthWest,
            _ => HatDirection::Neutral,
        };

        ParsedInput {
            report_id: buf[0],
            lx: buf[1],
            ly: buf[2],
            rx: buf[3],
            ry: buf[4],
            l2_axis: buf[8],
            r2_axis: buf[9],
            hat,
            face_buttons: FaceButtons::from_bits_truncate(buf[5]),
            shoulder_buttons: ShoulderButtons::from_bits_truncate(buf[6]),
            system_buttons: SystemButtons::from_bits_truncate(buf[7] & !SystemButtons::MUTE.bits()),
            ts: u16::from_le_bytes([buf[10], buf[11]]) as u32,
            battery_level: buf[12] & 0x0F,
            gx: i16::from_le_bytes([buf[13], buf[14]]),
            gy: i16::from_le_bytes([buf[15], buf[16]]),
            gz: i16::from_le_bytes([buf[17], buf[18]]),
            ax: i16::from_le_bytes([buf[19], buf[20]]),
            ay: i16::from_le_bytes([buf[21], buf[22]]),
            az: i16::from_le_bytes([buf[23], buf[24]]),
        }
    }

    pub fn from_ps5_buf(buf: &[u8; 64]) -> Self {
        let hat = match buf[8] & 0x0F {
            0x00 => HatDirection::North,
            0x01 => HatDirection::NorthEast,
            0x02 => HatDirection::East,
            0x03 => HatDirection::SouthEast,
            0x04 => HatDirection::South,
            0x05 => HatDirection::SouthWest,
            0x06 => HatDirection::West,
            0x07 => HatDirection::NorthWest,
            _ => HatDirection::Neutral,
        };

        ParsedInput {
            report_id: buf[0],
            lx: buf[1],
            ly: buf[2],
            rx: buf[3],
            ry: buf[4],
            l2_axis: buf[5],
            r2_axis: buf[6],
            hat,
            face_buttons: FaceButtons::from_bits_truncate(buf[8]),
            shoulder_buttons: ShoulderButtons::from_bits_truncate(buf[9]),
            system_buttons: SystemButtons::from_bits_truncate(buf[10]),
            battery_level: buf[63] & 0x0F,
            ts: u32::from_le_bytes([buf[31], buf[32], buf[33], buf[34]]),
            gx: i16::from_le_bytes([buf[19], buf[20]]),
            gy: i16::from_le_bytes([buf[21], buf[22]]),
            gz: i16::from_le_bytes([buf[23], buf[24]]),
            ax: i16::from_le_bytes([buf[25], buf[26]]),
            ay: i16::from_le_bytes([buf[27], buf[28]]),
            az: i16::from_le_bytes([buf[29], buf[30]]),
        }
    }
}

impl From<ParsedInput> for ControllerStateInternal {
    fn from(value: ParsedInput) -> Self {
        ControllerStateInternal {
            l: Axis2D {
                x: value.lx,
                y: value.ly,
            },
            r: Axis2D {
                x: value.rx,
                y: value.ry,
            },
            button: convert_all(
                value.face_buttons,
                value.shoulder_buttons,
                value.system_buttons,
                value.hat,
            ),
            l2_axis: value.l2_axis,
            r2_axis: value.r2_axis,
            gyro: Axis3D {
                x: value.gx,
                y: value.gy,
                z: value.gz,
            },
            accel: Axis3D {
                x: value.ax,
                y: value.ay,
                z: value.az,
            },
            ts: value.ts,
            battery: value.battery_level,
        }
    }
}

fn convert_face_buttons(face: FaceButtons) -> Buttons {
    let mut buttons = Buttons::empty();

    if face.contains(FaceButtons::SQUARE) {
        buttons |= Buttons::SQUARE;
    }
    if face.contains(FaceButtons::CROSS) {
        buttons |= Buttons::CROSS;
    }
    if face.contains(FaceButtons::CIRCLE) {
        buttons |= Buttons::CIRCLE;
    }
    if face.contains(FaceButtons::TRIANGLE) {
        buttons |= Buttons::TRIANGLE;
    }

    buttons
}

fn convert_shoulder_buttons(shoulder: ShoulderButtons) -> Buttons {
    let mut buttons = Buttons::empty();

    if shoulder.contains(ShoulderButtons::L1) {
        buttons |= Buttons::L1;
    }
    if shoulder.contains(ShoulderButtons::R1) {
        buttons |= Buttons::R1;
    }
    if shoulder.contains(ShoulderButtons::L2) {
        buttons |= Buttons::L2;
    }
    if shoulder.contains(ShoulderButtons::R2) {
        buttons |= Buttons::R2;
    }
    if shoulder.contains(ShoulderButtons::CREATE) {
        buttons |= Buttons::CREATE;
    }
    if shoulder.contains(ShoulderButtons::OPTIONS) {
        buttons |= Buttons::OPTIONS;
    }
    if shoulder.contains(ShoulderButtons::L3) {
        buttons |= Buttons::L3;
    }
    if shoulder.contains(ShoulderButtons::R3) {
        buttons |= Buttons::R3;
    }

    buttons
}

fn convert_system_buttons(system: SystemButtons) -> Buttons {
    let mut buttons = Buttons::empty();

    if system.contains(SystemButtons::PS) {
        buttons |= Buttons::PS;
    }
    if system.contains(SystemButtons::TOUCHPAD) {
        buttons |= Buttons::TOUCHPAD;
    }
    if system.contains(SystemButtons::MUTE) {
        buttons |= Buttons::MUTE;
    }

    buttons
}

fn convert_hat_direction(hat: HatDirection) -> Buttons {
    match hat {
        HatDirection::Neutral => Buttons::empty(),
        HatDirection::North => Buttons::HAT_UP,
        HatDirection::NorthEast => Buttons::HAT_UP | Buttons::HAT_RIGHT,
        HatDirection::East => Buttons::HAT_RIGHT,
        HatDirection::SouthEast => Buttons::HAT_DOWN | Buttons::HAT_RIGHT,
        HatDirection::South => Buttons::HAT_DOWN,
        HatDirection::SouthWest => Buttons::HAT_DOWN | Buttons::HAT_LEFT,
        HatDirection::West => Buttons::HAT_LEFT,
        HatDirection::NorthWest => Buttons::HAT_UP | Buttons::HAT_LEFT,
    }
}

fn convert_all(
    face: FaceButtons,
    shoulder: ShoulderButtons,
    system: SystemButtons,
    hat: HatDirection,
) -> Buttons {
    convert_face_buttons(face)
        | convert_shoulder_buttons(shoulder)
        | convert_system_buttons(system)
        | convert_hat_direction(hat)
}
