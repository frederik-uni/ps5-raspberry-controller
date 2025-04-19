use bitflags::bitflags;

bitflags! {
    #[derive(Debug)]
    pub struct Buttons: u32 {
        const SQUARE    = 1 << 0;
        const CROSS     = 1 << 1;
        const CIRCLE    = 1 << 2;
        const TRIANGLE  = 1 << 3;
        const HAT_UP    = 1 << 4;
        const HAT_DOWN  = 1 << 5;
        const HAT_LEFT  = 1 << 6;
        const HAT_RIGHT = 1 << 7;
        const L1        = 1 << 8;
        const R1        = 1 << 9;
        const L2        = 1 << 10;
        const R2        = 1 << 11;
        const CREATE    = 1 << 12;
        const OPTIONS   = 1 << 13;
        const L3        = 1 << 14;
        const R3        = 1 << 15;
        const PS        = 1 << 16;
        const TOUCHPAD  = 1 << 17;
        const MUTE      = 1 << 18;
    }
}

#[derive(Debug)]
pub struct Axis2D {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug)]
pub struct Axis3D {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Debug)]
pub struct ControllerStateInternal {
    pub l: Axis2D,
    pub r: Axis2D,
    pub gyro: Axis3D,
    pub accel: Axis3D,
    pub battery: u8,
    pub ts: u32,
    pub button: Buttons,
    pub l2_axis: u8,
    pub r2_axis: u8,
}
