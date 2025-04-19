use bitflags::bitflags;

bitflags! {
    pub struct ButtonsByte5: u8 {
        const SQUARE    = 1 << 4;
        const CROSS     = 1 << 5;
        const CIRCLE    = 1 << 6;
        const TRIANGLE  = 1 << 7;
    }
}

bitflags! {
    struct ButtonsByte6: u8 {
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
    struct ButtonsByte7: u8 {
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
