use std::fmt::Debug;

use crate::signal::SignalType;

#[derive(PartialEq)]
pub struct RawSignal {
    pub(crate) name: String,
    pub(crate) r#type: SignalType,
    pub(crate) frequency: u32,
    pub(crate) duty_cycle: f32,
    /// Data is a list of durations in microseconds.
    ///
    /// The first value is the duration of the first pulse, the second value is the duration of the
    /// pause after that, the third value is the duration of the second pulse, and so on.
    pub(crate) data: Vec<u32>,
}

impl Debug for RawSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SavedSignal")
            .field("name", &self.name)
            .field("type", &self.r#type)
            .field("frequency", &self.frequency)
            .field("duty_cycle", &self.duty_cycle)
            .finish_non_exhaustive()
    }
}

impl RawSignal {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data(&self) -> &[u32] {
        &self.data
    }
}
