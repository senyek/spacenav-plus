use libspnav_bindings as libspnav;
use std::convert::{From, Into, TryFrom};
use std::sync::Mutex;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy)]
pub enum EventType {
    Any,
    Motion,
    Button,
}

const SPNAV_EVENT_ANY: i32 = 0;
const SPNAV_EVENT_MOTION: i32 = 1;
const SPNAV_EVENT_BUTTON: i32 = 2;

impl Into<i32> for EventType {
    fn into(self) -> i32 {
        match self {
            EventType::Any => SPNAV_EVENT_ANY,
            EventType::Motion => SPNAV_EVENT_MOTION,
            EventType::Button => SPNAV_EVENT_BUTTON,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Motion(MotionEvent),
    Button(ButtonEvent),
}

#[derive(Debug, Clone)]
pub struct MotionEvent {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub rx: i32,
    pub ry: i32,
    pub rz: i32,
    pub period: u32,
}

impl MotionEvent {
    pub fn t(&self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }

    pub fn r(&self) -> (i32, i32, i32) {
        (self.rx, self.ry, self.rz)
    }
}

impl From<libspnav::spnav_event_motion> for MotionEvent {
    fn from(event: libspnav::spnav_event_motion) -> Self {
        MotionEvent {
            x: event.x,
            y: event.y,
            z: event.z,
            rx: event.rx,
            ry: event.ry,
            rz: event.rz,
            period: event.period,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ButtonEvent {
    pub press: bool,
    pub bnum: i32,
}

impl From<libspnav::spnav_event_button> for ButtonEvent {
    fn from(event: libspnav::spnav_event_button) -> Self {
        ButtonEvent {
            press: event.press != 0,
            bnum: event.bnum,
        }
    }
}

impl TryFrom<libspnav::spnav_event> for Event {
    type Error = ();
    fn try_from(event: libspnav::spnav_event) -> Result<Self, Self::Error> {
        unsafe {
            match event {
                libspnav::spnav_event {
                    type_: SPNAV_EVENT_MOTION,
                } => Ok(Event::Motion(event.motion.into())),
                libspnav::spnav_event {
                    type_: SPNAV_EVENT_BUTTON,
                } => Ok(Event::Button(event.button.into())),
                _ => Err(()),
            }
        }
    }
}

#[derive(Debug)]
pub struct Connection {
    pub fd: i32,
}

static CONN_COUNT: OnceLock<Mutex<usize>> = OnceLock::new();

impl Connection {
    pub fn new() -> Result<Connection, ()> {
        let conn_count = CONN_COUNT.get_or_init(|| Mutex::new(0));
        let mut count = conn_count.lock().expect("to lock");
        if *count > 0 {
            *count += 1;
            Ok(Connection {
                fd: lib::spnav_fd()?,
            })
        } else {
            *count = 1;
            lib::spnav_open()?;
            Ok(Connection {
                fd: lib::spnav_fd()?,
            })
        }
    }

    pub fn poll(&self) -> Option<Event> {
        lib::spnav_poll_event()
    }

    pub fn wait(&self) -> Result<Event, ()> {
        lib::spnav_wait_event()
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if let Some(conn_count) = CONN_COUNT.get() {
            let mut count = conn_count.lock().expect("to lock");
            if *count == 1 {
                *count = 0;
                lib::spnav_close().expect("to close");
            } else {
                *count -= 1;
            }
        }
    }
}

pub mod lib {
    use super::*;

    pub fn spnav_open() -> Result<(), ()> {
        unsafe {
            if libspnav::spnav_open() == -1 {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    pub fn spnav_close() -> Result<(), ()> {
        unsafe {
            if libspnav::spnav_close() == -1 {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    pub fn spnav_fd() -> Result<i32, ()> {
        unsafe {
            let fd = libspnav::spnav_fd();
            if fd == -1 {
                Err(())
            } else {
                Ok(fd)
            }
        }
    }

    pub fn spnav_sensitivity(sens: f64) -> Result<i32, ()> {
        unsafe {
            let v = libspnav::spnav_sensitivity(sens);
            if v == -1 {
                Err(())
            } else {
                Ok(v)
            }
        }
    }

    pub fn spnav_wait_event() -> Result<Event, ()> {
        let mut event = libspnav::spnav_event {
            type_: SPNAV_EVENT_ANY,
        };
        let t = unsafe { libspnav::spnav_wait_event(&mut event) };
        if t == 0 {
            Err(())
        } else {
            event.try_into()
        }
    }

    pub fn spnav_poll_event() -> Option<Event> {
        let mut event = libspnav::spnav_event {
            type_: SPNAV_EVENT_ANY,
        };
        let t = unsafe { libspnav::spnav_poll_event(&mut event) };
        if t == 0 {
            None
        } else {
            event.try_into().ok()
        }
    }

    pub fn spnav_remove_events(t: EventType) -> i32 {
        unsafe { libspnav::spnav_remove_events(t.into()) }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() -> Result<(), ()> {
        let c = Connection::new()?;
        println!("{:?}", c);
        println!("{:?}", c.wait());
        Ok(())
    }
}
