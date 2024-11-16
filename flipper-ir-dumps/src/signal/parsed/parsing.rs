use displaydoc::Display;
use flipper_utils::round_to;
use nom::{combinator::all_consuming, multi::many1, Finish, IResult};
use thiserror::Error;

use super::Packet;

#[derive(Debug, PartialEq, Eq)]
enum DurationClass {
    /// A short duration (~550ms in case of my Samsung devices).
    Short,
    /// A long duration (typically ~3x the short duration).
    Long,
    /// Unusual duration.
    Unusual(u32),
}

#[derive(Debug, PartialEq, Eq)]
enum SignalComponent {
    Pulse,
    Pause,
}

#[derive(Debug, PartialEq, Eq)]
struct TimeSlot {
    duration: DurationClass,
    component: SignalComponent,
}

const SHORT_DURATION: u32 = 550;
const LONG_BIT_DURATION: u32 = 3 * SHORT_DURATION;

const ROUND_TO: u32 = SHORT_DURATION;

#[derive(Debug, Display, Error)]
/// Error parsing IR signals
pub enum ParseError {
    /// Nom error: {0}
    Nom(String),
}

pub(super) fn stream_to_packets(signal_timings: &[u32]) -> Result<Vec<Packet>, ParseError> {
    let signals = stream_to_signals(signal_timings);
    let (_, packets) = ir_dump_to_packets(&signals)
        .finish()
        .map_err(|e| ParseError::Nom(format!("{:?}", e)))?;

    Ok(packets)
}

fn ir_dump_to_packets(stream: &[TimeSlot]) -> IResult<&[TimeSlot], Vec<Packet>> {
    let (signals, _) = ir_dump_start(stream)?;
    let (signals, packets) = all_consuming(many1(single_packet))(signals)?;

    Ok((signals, packets))
}

fn stream_to_signals(signal_timings: &[u32]) -> Vec<TimeSlot> {
    signal_timings
        .iter()
        .enumerate()
        .map(|(i, &duration)| {
            (
                if i & 1 == 0 {
                    SignalComponent::Pulse
                } else {
                    SignalComponent::Pause
                },
                duration,
            )
        })
        .map(|(component, duration)| {
            let duration = match round_to(duration, ROUND_TO) {
                SHORT_DURATION => DurationClass::Short,
                LONG_BIT_DURATION => DurationClass::Long,
                _ => DurationClass::Unusual(duration),
            };
            TimeSlot {
                duration,
                component,
            }
        })
        .collect()
}

macro_rules! ts {
    (+short) => {
        TimeSlot {
            duration: DurationClass::Short,
            component: SignalComponent::Pulse,
        }
    };
    (-short) => {
        TimeSlot {
            duration: DurationClass::Short,
            component: SignalComponent::Pause,
        }
    };
    (+long) => {
        TimeSlot {
            duration: DurationClass::Long,
            component: SignalComponent::Pulse,
        }
    };
    (-long) => {
        TimeSlot {
            duration: DurationClass::Long,
            component: SignalComponent::Pause,
        }
    };
    (+$value:ident) => {
        TimeSlot {
            duration: DurationClass::Unusual($value),
            component: SignalComponent::Pulse,
        }
    };
    (-$value:ident) => {
        TimeSlot {
            duration: DurationClass::Unusual($value),
            component: SignalComponent::Pause,
        }
    };
    (+$value:literal) => {
        TimeSlot {
            duration: DurationClass::Unusual($value),
            component: SignalComponent::Pulse,
        }
    };
    (-$value:literal) => {
        TimeSlot {
            duration: DurationClass::Unusual($value),
            component: SignalComponent::Pause,
        }
    };
}

/// Dump starts with a single short pulse, followed by a "super-long"
/// (something like 17700ns) pause.
fn ir_dump_start(stream: &[TimeSlot]) -> IResult<&[TimeSlot], ()> {
    match stream {
        [] => Err(nom::Err::Error(nom::error::Error::new(
            stream,
            nom::error::ErrorKind::Eof,
        ))),
        [ts!(+short), ts!(-x), rest @ ..] if x / SHORT_DURATION > 26 => Ok((rest, ())),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            stream,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// A single bit is encoded as a short pulse followed
/// by either a short pause (0) or a long pause (1).
fn packet_bit(stream: &[TimeSlot]) -> IResult<&[TimeSlot], bool> {
    match stream {
        [] => Err(nom::Err::Error(nom::error::Error::new(
            stream,
            nom::error::ErrorKind::Eof,
        ))),
        [ts!(+short), ts!(-short), rest @ ..] => Ok((rest, false)),
        [ts!(+short), ts!(-long), rest @ ..] => Ok((rest, true)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            stream,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// Each signal starts as an unusually long (~3000ns) pulse, followed by
/// an unusually long (~9000ns) pause.
fn packet_start(stream: &[TimeSlot]) -> IResult<&[TimeSlot], ()> {
    match stream {
        [] => Err(nom::Err::Error(nom::error::Error::new(
            stream,
            nom::error::ErrorKind::Eof,
        ))),
        [ts!(+pulse), ts!(-pause), rest @ ..]
            if (4..7).contains(&(pulse / SHORT_DURATION))
                && (15..20).contains(&(pause / SHORT_DURATION)) =>
        {
            Ok((rest, ()))
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            stream,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// Packet ends with a single short pulse and either a ~3000ns pause,
/// or nothing (if this is the last packet).
fn packet_end(stream: &[TimeSlot]) -> IResult<&[TimeSlot], ()> {
    match stream {
        [] => Err(nom::Err::Error(nom::error::Error::new(
            stream,
            nom::error::ErrorKind::Eof,
        ))),
        // either a short pulse followed by the end of the stream
        [ts!(+short)] => Ok((&stream[1..], ())),
        // or a short pulse followed by long (~3000ns) pause
        [ts!(+short), ts!(-pause), rest @ ..] if (4..7).contains(&(pause / SHORT_DURATION)) => {
            Ok((rest, ()))
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            stream,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// A single packet is encoded as a start signal, followed by a stream of bits.
fn single_packet(stream: &[TimeSlot]) -> IResult<&[TimeSlot], Packet> {
    let (stream, _) = packet_start(stream)?;
    let (stream, bits) = many1(packet_bit)(stream)?;
    let (stream, _) = packet_end(stream)?;

    let mut packet = Packet::default();
    for bit in bits.iter().rev() {
        packet.data.push(*bit);
    }

    Ok((stream, packet))
}

#[cfg(test)]
mod tests {
    use bitvec::{bits, order::Lsb0, vec::BitVec};

    use super::*;

    #[test]
    fn test_ir_dump_start() {
        let stream = vec![ts!(+short), ts!(-short)];
        assert!(ir_dump_start(&stream).is_err());

        let stream = vec![ts!(+short), ts!(-17700)];
        assert_eq!(ir_dump_start(&stream), Ok((&[][..], ())));

        let stream = vec![ts!(+short), ts!(+17700)];
        assert!(ir_dump_start(&stream).is_err());
    }

    #[test]
    fn test_packet_start() {
        let stream = vec![ts!(+short), ts!(-short)];
        assert!(packet_start(&stream).is_err());

        let stream = vec![ts!(+short), ts!(-short), ts!(+short), ts!(-short)];
        assert!(packet_start(&stream).is_err());

        let stream = vec![ts!(+2972), ts!(-8930)];
        assert_eq!(packet_start(&stream), Ok((&[][..], ())));
    }

    #[test]
    fn test_signal_bit() {
        let stream = vec![ts!(+short), ts!(-short)];
        assert_eq!(packet_bit(&stream), Ok((&[][..], false)));

        let stream = vec![ts!(+short), ts!(-long)];
        assert_eq!(packet_bit(&stream), Ok((&[][..], true)));

        let stream = vec![ts!(+short), ts!(-short), ts!(+short), ts!(-long)];
        assert_eq!(
            packet_bit(&stream),
            Ok((&[ts!(+short), ts!(-long)][..], false))
        );
    }

    #[test]
    fn test_single_packet() {
        let stream = vec![
            ts!(+2972),
            ts!(-8930),
            ts!(+short),
            ts!(-short),
            ts!(+short),
            ts!(-long),
            ts!(+short),
        ];

        assert_eq!(
            single_packet(&stream),
            Ok((
                &[][..],
                // packet bits transmitted in LSB order, so 01 in the stream is 10 in the packet
                Packet {
                    data: BitVec::from_bitslice(bits![1, 0])
                }
            ))
        );
    }

    #[test]
    fn test_ir_dump() {
        let stream = vec![
            // dump header
            ts!(+short),
            ts!(-17700),
            // packet header
            ts!(+2972),
            ts!(-8930),
            // 0 bit
            ts!(+short),
            ts!(-short),
            // 1 bit
            ts!(+short),
            ts!(-long),
            // packet end (with pause due to next packet)
            ts!(+short),
            ts!(-2920),
            // packet header
            ts!(+2972),
            ts!(-8930),
            // 1 bit
            ts!(+short),
            ts!(-long),
            // 0 bit
            ts!(+short),
            ts!(-short),
            // packet end (last packet)
            ts!(+short),
            // dump end
        ];

        assert_eq!(
            ir_dump_to_packets(&stream),
            Ok((
                &[][..],
                vec![
                    // packet bits transmitted in LSB order, so 01 in the stream is 10 in the packet
                    Packet {
                        data: BitVec::from_bitslice(bits![1, 0])
                    },
                    Packet {
                        data: BitVec::from_bitslice(bits![0, 1])
                    }
                ]
            ))
        );
    }
}
