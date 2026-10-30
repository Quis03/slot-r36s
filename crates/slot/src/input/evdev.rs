use std::path::{Path, PathBuf};

use slot_input::{Btn, RawEvent};

pub const EVENT_BYTES: usize = 24;

pub const EV_SYN: u16 = 0x00;
pub const EV_KEY: u16 = 0x01;
pub const EV_SW: u16 = 0x05;
pub const SW_LID: u16 = 0x00;

pub const EV_ABS: u16 = 0x03;
pub const ABS_HAT0X: u16 = 0x10;
pub const ABS_HAT0Y: u16 = 0x11;
pub const ABS_LSX: u16 = 0x02;
pub const ABS_LSY: u16 = 0x03;
pub const ABS_X: u16 = 0x00;
pub const ABS_Y: u16 = 0x01;

pub const STICK_DEADZONE: i32 = 2048;

pub const ODROIDGO3_PAD: &str = "GO-Super Gamepad";

const ODROIDGO3_DEADZONE_PERCENT: i32 = 40;

const LEFT_STICK_TARGETS: [&str; 7] = [
    "r36s",
    "rg34xxsp",
    "rg35xxh",
    "rg35xxpro",
    "rg40xxh",
    "rg40xxv",
    "rgcubexx",
];

pub fn has_left_stick(target: &str) -> bool {
    LEFT_STICK_TARGETS.contains(&target)
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Layout {
    #[default]
    Anbernic,
    OdroidGo3,
}

impl Layout {
    pub fn for_name(name: &str) -> Layout {
        match name.trim() {
            ODROIDGO3_PAD => Layout::OdroidGo3,
            _ => Layout::Anbernic,
        }
    }

    pub fn stick_axes(self) -> (u16, u16) {
        match self {
            Layout::Anbernic => (ABS_LSX, ABS_LSY),
            Layout::OdroidGo3 => (ABS_X, ABS_Y),
        }
    }

    pub fn stick_deadzone(self, axis_max: Option<i32>) -> i32 {
        match (self, axis_max) {
            (Layout::OdroidGo3, Some(max)) if max > 0 => max * ODROIDGO3_DEADZONE_PERCENT / 100,
            _ => STICK_DEADZONE,
        }
    }
}

pub struct Hat {
    hat: [i32; 2],
    stick: [i32; 2],
    steers: bool,
    axes: (u16, u16),
    deadzone: i32,
}

impl Default for Hat {
    fn default() -> Self {
        Hat::for_layout(Layout::Anbernic, STICK_DEADZONE)
    }
}

impl Hat {
    pub fn for_layout(layout: Layout, deadzone: i32) -> Self {
        Hat {
            hat: [0; 2],
            stick: [0; 2],
            steers: false,
            axes: layout.stick_axes(),
            deadzone,
        }
    }

    pub fn steer_with_stick(&mut self, on: bool) {
        self.steers = on;
    }

    pub fn feed(&mut self, ev: Ev) -> Vec<RawEvent> {
        let dead = self.deadzone;
        let tilt = |v: i32| match v {
            v if v > dead => 1,
            v if v < -dead => -1,
            _ => 0,
        };
        let steer = |v: i32| if self.steers { tilt(v) } else { 0 };
        let (axis, from_stick, value) = match ev.code {
            ABS_HAT0X => (0, false, ev.value.signum()),
            ABS_HAT0Y => (1, false, ev.value.signum()),
            c if c == self.axes.0 => (0, true, steer(ev.value)),
            c if c == self.axes.1 => (1, true, steer(ev.value)),
            _ => return Vec::new(),
        };
        let was = self.held(axis);
        match from_stick {
            true => self.stick[axis] = value,
            false => self.hat[axis] = value,
        }
        let now = self.held(axis);
        if now == was {
            return Vec::new();
        }
        let ends = [[Btn::Left, Btn::Right], [Btn::Up, Btn::Down]][axis];
        let btn = |v: i32| ends[usize::from(v > 0)];
        let mut out = Vec::new();
        if was != 0 {
            out.push(RawEvent::Up(btn(was)));
        }
        if now != 0 {
            out.push(RawEvent::Down(btn(now)));
        }
        out
    }

    fn held(&self, axis: usize) -> i32 {
        match self.hat[axis] {
            0 => self.stick[axis],
            hat => hat,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Ev {
    pub kind: u16,
    pub code: u16,
    pub value: i32,
}

pub fn decode(bytes: &[u8]) -> Option<Ev> {
    let packet: &[u8; EVENT_BYTES] = bytes.get(..EVENT_BYTES)?.try_into().ok()?;
    Some(Ev {
        kind: u16::from_le_bytes([packet[16], packet[17]]),
        code: u16::from_le_bytes([packet[18], packet[19]]),
        value: i32::from_le_bytes([packet[20], packet[21], packet[22], packet[23]]),
    })
}

pub fn to_raw(ev: Ev) -> Option<RawEvent> {
    to_raw_in(Layout::Anbernic, ev)
}

pub fn to_raw_in(layout: Layout, ev: Ev) -> Option<RawEvent> {
    let btn = match ev.kind {
        EV_KEY => code_to_btn_in(layout, ev.code)?,
        EV_SW if ev.code == SW_LID => Btn::Lid,
        _ => return None,
    };
    match ev.value {
        0 => Some(RawEvent::Up(btn)),
        1 => Some(RawEvent::Down(btn)),
        _ => None,
    }
}

pub fn code_to_btn_in(layout: Layout, code: u16) -> Option<Btn> {
    match layout {
        Layout::Anbernic => code_to_btn(code),
        Layout::OdroidGo3 => odroidgo3_btn(code),
    }
}

fn odroidgo3_btn(code: u16) -> Option<Btn> {
    Some(match code {
        0x220 => Btn::Up,
        0x221 => Btn::Down,
        0x222 => Btn::Left,
        0x223 => Btn::Right,
        0x131 => Btn::A,
        0x130 => Btn::B,
        0x133 => Btn::X,
        0x134 => Btn::Y,
        0x136 => Btn::L1,
        0x137 => Btn::R1,
        0x138 => Btn::L2,
        0x139 => Btn::R2,
        0x2c0 => Btn::Select,
        0x2c1 => Btn::Start,
        0x2c4 => Btn::Menu,
        115 => Btn::VolUp,
        114 => Btn::VolDown,
        116 => Btn::Power,
        _ => return None,
    })
}

pub fn code_to_btn(code: u16) -> Option<Btn> {
    Some(match code {
        0x130 => Btn::A,
        0x131 => Btn::B,
        0x132 => Btn::Y,
        0x133 => Btn::X,
        0x134 => Btn::L1,
        0x135 => Btn::R1,
        0x136 => Btn::Select,
        0x137 => Btn::Start,
        0x138 => Btn::Menu,
        0x13a => Btn::L2,
        0x13b => Btn::R2,
        115 => Btn::VolUp,
        114 => Btn::VolDown,
        116 => Btn::Power,
        _ => return None,
    })
}

const WANTED: [u16; 17] = [
    0x130, 0x131, 0x132, 0x133, 0x134, 0x135, 0x136, 0x137, 0x138, 0x13a, 0x13b, 0x162, 0x220,
    0x2c4, 115, 114, 116,
];

pub use slot_power::has_bit;

#[cfg(unix)]
extern "C" {
    fn ioctl(fd: std::ffi::c_int, request: std::ffi::c_ulong, ...) -> std::ffi::c_int;
}

#[cfg(unix)]
const EVIOCGABS: std::ffi::c_ulong = 0x8018_4540;

#[cfg(unix)]
pub fn abs_max(node: &std::fs::File, axis: u16) -> Option<i32> {
    use std::os::unix::io::AsRawFd;
    let mut info = [0i32; 6];
    let request = EVIOCGABS + std::ffi::c_ulong::from(axis);
    let rc = unsafe { ioctl(node.as_raw_fd(), request, info.as_mut_ptr()) };
    (rc == 0).then(|| info[1].abs().max(info[2].abs()))
}

#[cfg(not(unix))]
pub fn abs_max(_node: &std::fs::File, _axis: u16) -> Option<i32> {
    None
}

pub fn pick_devices(dev: &Path, sys: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dev) else {
        return Vec::new();
    };
    let mut nodes: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("event"))
        })
        .collect();
    nodes.sort();
    nodes.retain(|node| {
        let Some(name) = node.file_name() else {
            return false;
        };
        wanted_node(&sys.join(name).join("device"))
    });
    nodes
}

fn wanted_node(device: &Path) -> bool {
    let cap = |file: &str| std::fs::read_to_string(device.join("capabilities").join(file));
    let keys = cap("key").unwrap_or_default();
    let lid = cap("sw").is_ok_and(|sw| has_bit(&sw, SW_LID));
    lid || WANTED.iter().any(|bit| has_bit(&keys, *bit))
}

pub fn device_name(sys: &Path, node: &Path) -> String {
    node.file_name()
        .and_then(|n| std::fs::read_to_string(sys.join(n).join("device/name")).ok())
        .map(|n| n.trim().to_string())
        .unwrap_or_else(|| node.display().to_string())
}
