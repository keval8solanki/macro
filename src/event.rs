use rdev::{Button, Event, EventType, Key};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableEvent {
    pub event_type: SerializableEventType,
    pub delay_ms: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SerializableEventType {
    KeyPress(Key),
    KeyRelease(Key),
    ButtonPress(Button),
    ButtonRelease(Button),
    MouseMove { x: f64, y: f64 },
    Wheel { delta_x: i64, delta_y: i64 },
}

impl SerializableEvent {
    pub fn from_rdev(event: Event, delay_ms: u64) -> Option<Self> {
        let event_type = match event.event_type {
            EventType::KeyPress(key) => SerializableEventType::KeyPress(key),
            EventType::KeyRelease(key) => SerializableEventType::KeyRelease(key),
            EventType::ButtonPress(btn) => SerializableEventType::ButtonPress(btn),
            EventType::ButtonRelease(btn) => SerializableEventType::ButtonRelease(btn),
            EventType::MouseMove { x, y } => SerializableEventType::MouseMove { x, y },
            EventType::Wheel { delta_x, delta_y } => SerializableEventType::Wheel { delta_x, delta_y },
        };
        Some(Self {
            event_type,
            delay_ms,
        })
    }

    pub fn to_rdev(&self) -> EventType {
        match self.event_type {
            SerializableEventType::KeyPress(key) => EventType::KeyPress(key),
            SerializableEventType::KeyRelease(key) => EventType::KeyRelease(key),
            SerializableEventType::ButtonPress(btn) => EventType::ButtonPress(btn),
            SerializableEventType::ButtonRelease(btn) => EventType::ButtonRelease(btn),
            SerializableEventType::MouseMove { x, y } => EventType::MouseMove { x, y },
            SerializableEventType::Wheel { delta_x, delta_y } => EventType::Wheel { delta_x, delta_y },
        }
    }
}
