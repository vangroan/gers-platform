pub enum EventType {
    NoOp = 0,
    Hello = 1,
}

impl From<i32> for EventType {
    fn from(value: i32) -> EventType {
        match value {
            1 => Self::Hello,
            _ => Self::NoOp,
        }
    }
}

/// Data for `Hello` event.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct HelloEvent {
    pub data: u32,
    pub padding: u8,
    pub div: u16,
}
