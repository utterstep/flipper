use std::fmt::Debug;

use bitvec::{order::Lsb0, vec::BitVec};

use crate::signal::SignalType;

use super::RawSignal;

mod parsing;
use parsing::{stream_to_packets, ParseError};

type DataVec = BitVec<usize, Lsb0>;

#[derive(Debug)]
pub struct ParsedSignal {
    pub(crate) name: String,
    pub(crate) r#type: SignalType,
    pub(crate) frequency: u32,
    pub(crate) duty_cycle: f32,
    pub(crate) packets: Vec<Packet>,
}

impl ParsedSignal {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn r#type(&self) -> SignalType {
        self.r#type
    }

    pub fn frequency(&self) -> u32 {
        self.frequency
    }

    pub fn duty_cycle(&self) -> f32 {
        self.duty_cycle
    }

    pub fn packets(&self) -> &[Packet] {
        &self.packets
    }
}

#[derive(Default, PartialEq, Eq)]
pub struct Packet {
    pub(crate) data: DataVec,
}

impl std::fmt::Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // prints out the data as a series of 0s and 1s
        for bit in self.data.iter() {
            write!(f, "{}", if *bit { '1' } else { '0' })?;
        }
        Ok(())
    }
}

impl Debug for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // prints out the data as a series of 0s and 1s
        f.debug_struct("Packet")
            .field("data", &format!("{}", self))
            .finish()
    }
}

impl TryFrom<&RawSignal> for ParsedSignal {
    type Error = ParseError;

    fn try_from(raw: &RawSignal) -> Result<Self, Self::Error> {
        let packets = stream_to_packets(&raw.data)?;

        Ok(ParsedSignal {
            name: raw.name.clone(),
            r#type: raw.r#type,
            frequency: raw.frequency,
            duty_cycle: raw.duty_cycle,
            packets,
        })
    }
}
