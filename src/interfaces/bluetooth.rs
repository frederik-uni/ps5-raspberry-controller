use bitflags::bitflags;

use super::internal::{Buttons, ControllerStateInternal};

bitflags! {
    #[derive(Debug, Clone, Copy, Default)]
    pub struct ButtonsByte5: u8 {
        const SQUARE    = 1 << 4;
        const CROSS     = 1 << 5;
        const CIRCLE    = 1 << 6;
        const TRIANGLE  = 1 << 7;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, Default)]
    pub struct ButtonsByte6: u8 {
        const L1         = 1 << 0;
        const R1         = 1 << 1;
        const L2         = 1 << 2;
        const R2         = 1 << 3;
        const CREATE     = 1 << 4;
        const OPTIONS    = 1 << 5;
        const L3         = 1 << 6;
        const R3         = 1 << 7;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, Default)]
    pub struct ButtonsByte7: u8 {
        const PS         = 1 << 0;
        const TOUCHPAD   = 1 << 1;
        // Remaining 6 bits are vendor-defined and can be set as a raw value
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ControllerState {
    pub left_stick_x: u8,
    pub left_stick_y: u8,
    pub right_stick_x: u8,
    pub right_stick_y: u8,
    pub hat: u8, // 0x0 - 0x7 for directions, 0x8 for neutral
    pub buttons_5: ButtonsByte5,
    pub buttons_6: ButtonsByte6,
    pub buttons_7: ButtonsByte7,
    pub vendor_defined: u8, // Only lower 6 bits used
    pub l2_axis: u8,
    pub r2_axis: u8,
}

fn convert_from_buttons(buttons: Buttons) -> (ButtonsByte5, ButtonsByte6, ButtonsByte7, u8) {
    let mut byte5 = ButtonsByte5::empty();
    let mut byte6 = ButtonsByte6::empty();
    let mut byte7 = ButtonsByte7::empty();

    // Byte 5 (face buttons)
    if buttons.contains(Buttons::SQUARE) {
        byte5 |= ButtonsByte5::SQUARE;
    }
    if buttons.contains(Buttons::CROSS) {
        byte5 |= ButtonsByte5::CROSS;
    }
    if buttons.contains(Buttons::CIRCLE) {
        byte5 |= ButtonsByte5::CIRCLE;
    }
    if buttons.contains(Buttons::TRIANGLE) {
        byte5 |= ButtonsByte5::TRIANGLE;
    }

    // Byte 6 (shoulders + misc)
    if buttons.contains(Buttons::L1) {
        byte6 |= ButtonsByte6::L1;
    }
    if buttons.contains(Buttons::R1) {
        byte6 |= ButtonsByte6::R1;
    }
    if buttons.contains(Buttons::L2) {
        byte6 |= ButtonsByte6::L2;
    }
    if buttons.contains(Buttons::R2) {
        byte6 |= ButtonsByte6::R2;
    }
    if buttons.contains(Buttons::CREATE) {
        byte6 |= ButtonsByte6::CREATE;
    }
    if buttons.contains(Buttons::OPTIONS) {
        byte6 |= ButtonsByte6::OPTIONS;
    }
    if buttons.contains(Buttons::L3) {
        byte6 |= ButtonsByte6::L3;
    }
    if buttons.contains(Buttons::R3) {
        byte6 |= ButtonsByte6::R3;
    }

    // Byte 7 (PS & touchpad)
    if buttons.contains(Buttons::PS) {
        byte7 |= ButtonsByte7::PS;
    }
    if buttons.contains(Buttons::TOUCHPAD) {
        byte7 |= ButtonsByte7::TOUCHPAD;
    }

    // Hat (D-pad)
    let up = buttons.contains(Buttons::HAT_UP);
    let down = buttons.contains(Buttons::HAT_DOWN);
    let left = buttons.contains(Buttons::HAT_LEFT);
    let right = buttons.contains(Buttons::HAT_RIGHT);

    let hat = match (up, down, left, right) {
        (true, false, false, false) => 7,
        (true, false, true, false) => 6,
        (false, false, true, false) => 5,
        (false, true, true, false) => 4,
        (false, true, false, false) => 3,
        (false, true, false, true) => 2,
        (false, false, false, true) => 1,
        (true, false, false, true) => 0,
        _ => 8, // neutral or invalid combo
    };

    (byte5, byte6, byte7, hat)
}

impl From<ControllerStateInternal> for ControllerState {
    fn from(value: ControllerStateInternal) -> Self {
        let (b5, b6, b7, h) = convert_from_buttons(value.button);
        ControllerState {
            left_stick_x: value.l.x,
            left_stick_y: value.l.y,
            right_stick_x: value.r.x,
            right_stick_y: value.r.y,
            hat: h,
            buttons_5: b5,
            buttons_6: b6,
            buttons_7: b7,
            vendor_defined: 0 & 0x3F,
            l2_axis: value.l2_axis,
            r2_axis: value.r2_axis,
        }
    }
}

impl ControllerState {
    pub fn to_bytes(&self) -> [u8; 10] {
        let mut byte5 = self.hat & 0x0F; // lower 4 bits for HAT
        byte5 |= self.buttons_5.bits();

        let byte6 = self.buttons_6.bits();

        let mut byte7 = self.buttons_7.bits() & 0x03; // PS and Touchpad
        byte7 |= (self.vendor_defined & 0x3F) << 2;

        [
            0x01,               // Report ID
            self.left_stick_x,  // Left stick X
            self.left_stick_y,  // Left stick Y
            self.right_stick_x, // Right stick X
            self.right_stick_y, // Right stick Y
            byte5,              // Hat + face buttons
            byte6,              // Shoulder + meta buttons
            byte7,              // PS, Touchpad, Vendor-defined
            self.l2_axis,       // L2 analog axis
            self.r2_axis,       // R2 analog axis
        ]
    }
}
