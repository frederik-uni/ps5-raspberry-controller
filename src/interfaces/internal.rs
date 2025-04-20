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

bitflags! {
    #[derive(Debug)]
    pub struct PowerState: u8 {
        const Discharging         = 0x00; // Use PowerPercent
        const Charging            = 0x01; // Use PowerPercent
        const Complete            = 0x02; // PowerPercent not valid? assume 100%?
        const AbnormalVoltage     = 0x0A; // PowerPercent not valid?
        const AbnormalTemperature = 0x0B; // PowerPercent not valid?
        const ChargingError       = 0x0F; // PowerPercent not valid?
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
    pub power_state: PowerState,
    pub ts: u32,
    pub button: Buttons,
    pub l2_axis: u8,
    pub r2_axis: u8,
}

pub struct Profile {
    pub name: String,
    pub sensitivity: String,
    pub macros: Vec<Macro>,
}

pub struct Macro {
    //input
    include: Buttons,
    exclude: Buttons,
    //output
    filter: Buttons,
    add: Buttons,
    switch_profile: Option<String>,
    hold_sensitivity: Option<String>,
    macro_list: Option<Vec<MacroAction>>,
}

enum MacroAction {
    PressJoystick { left: Axis2D, right: Axis2D },
    Press(Buttons),
    ReleaseJoystick { left: Axis2D, right: Axis2D },
    Release(Buttons),
    Sleep(u64),
}

pub struct SensitivityProfile {
    pub name: String,
    pub curve: Vec<Point>,
}

impl SensitivityProfile {
    pub fn get(&self, x: u8) -> Point {
        if let Some(p) = self.curve.iter().find(|p| p.x == x) {
            return *p;
        }

        // should never happen
        if x < self.curve.first().unwrap().x || x > self.curve.last().unwrap().x {
            return Point { x, y: x };
        }

        for window in self.curve.windows(2) {
            let a = window[0];
            let b = window[1];
            if x > a.x && x < b.x {
                let m = (b.y as f64 - a.y as f64) / (b.x as f64 - a.x as f64);
                let y = a.y as f64 + m * (x as f64 - a.x as f64);
                return Point {
                    x,
                    y: y.round() as u8,
                };
            }
        }

        // should never happen
        Point { x, y: x }
    }
}

#[derive(Clone, Copy)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}
