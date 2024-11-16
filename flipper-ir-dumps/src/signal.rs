mod parsed;
mod raw;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SignalType {
    Raw,
}

pub use parsed::{Packet, ParsedSignal};
pub use raw::RawSignal;
