use std::fmt::Debug;

use nom::{
    bytes::complete::tag,
    character::complete::{digit1, line_ending, not_line_ending},
    combinator::{all_consuming, map_res},
    multi::{many0, separated_list0},
    number, Finish, Parser,
};

#[derive(Debug, PartialEq)]
pub struct DumpFile {
    version: u32,
    signals: Vec<SavedSignal>,
}

impl DumpFile {
    pub fn signals(&self) -> &[SavedSignal] {
        &self.signals
    }
}

impl<'a> TryFrom<&'a str> for DumpFile {
    type Error = nom::error::Error<&'a str>;

    fn try_from(input: &'a str) -> Result<Self, Self::Error> {
        dump_file(input).finish().map(|(_, dump)| dump)
    }
}

fn dump_file(input: &str) -> nom::IResult<&str, DumpFile> {
    let (input, _) = tag("Filetype: IR signals file")(input)?;
    let (input, _) = line_ending(input)?;

    let (input, version) = version(input)?;
    let (input, _) = line_ending(input)?;

    let (input, signals) = all_consuming(many0(saved_signal))(input)?;

    Ok((input, DumpFile { version, signals }))
}

fn version(input: &str) -> nom::IResult<&str, u32> {
    let (input, _) = tag("Version: ")(input)?;
    let (input, version) = digit1(input)?;

    Ok((input, version.parse().unwrap()))
}

#[derive(PartialEq)]
pub struct SavedSignal {
    name: String,
    r#type: SignalType,
    frequency: u32,
    duty_cycle: f32,
    /// Data is a list of durations in microseconds.
    ///
    /// The first value is the duration of the first pulse, the second value is the duration of the
    /// pause after that, the third value is the duration of the second pulse, and so on.
    data: Vec<u32>,
}

impl Debug for SavedSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SavedSignal")
            .field("name", &self.name)
            .field("type", &self.r#type)
            .field("frequency", &self.frequency)
            .field("duty_cycle", &self.duty_cycle)
            .finish_non_exhaustive()
    }
}

impl SavedSignal {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data(&self) -> &[u32] {
        &self.data
    }
}

fn saved_signal(input: &str) -> nom::IResult<&str, SavedSignal> {
    let (input, _) = tag("#")(input)?;
    let (input, _) = not_line_ending(input)?;
    let (input, _) = line_ending(input)?;

    let (input, name) = name(input)?;
    let (input, _) = line_ending(input)?;

    let (input, r#type) = signal_type(input)?;
    let (input, _) = line_ending(input)?;

    let (input, frequency) = frequency(input)?;
    let (input, _) = line_ending(input)?;

    let (input, duty_cycle) = duty_cycle(input)?;
    let (input, _) = line_ending(input)?;

    let (input, data) = data(input)?;
    let (input, _) = line_ending(input)?;

    Ok((
        input,
        SavedSignal {
            name,
            r#type,
            frequency,
            duty_cycle,
            data,
        },
    ))
}

fn name(input: &str) -> nom::IResult<&str, String> {
    let (input, _) = tag("name: ")(input)?;
    let (input, name) = not_line_ending(input)?;

    Ok((input, name.to_string()))
}

fn frequency(input: &str) -> nom::IResult<&str, u32> {
    let (input, _) = tag("frequency: ")(input)?;
    let (input, frequency) = parse_u32_str(input)?;

    Ok((input, frequency))
}

fn duty_cycle(input: &str) -> nom::IResult<&str, f32> {
    let (input, _) = tag("duty_cycle: ")(input)?;
    let (input, duty_cycle) = number::complete::float(input)?;

    Ok((input, duty_cycle))
}

fn data(input: &str) -> nom::IResult<&str, Vec<u32>> {
    let (input, _) = tag("data: ")(input)?;
    let (input, data) = separated_list0(tag(" "), parse_u32_str)(input)?;

    Ok((input, data))
}

fn parse_u32_str(input: &str) -> nom::IResult<&str, u32> {
    map_res(digit1, |input: &str| input.parse::<u32>()).parse(input)
}

#[derive(Debug, PartialEq)]
pub enum SignalType {
    Raw,
}

fn signal_type(input: &str) -> nom::IResult<&str, SignalType> {
    let (input, _) = tag("type: ")(input)?;
    tag("raw")(input).map(|(input, _)| (input, SignalType::Raw))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_version() {
        let input = "Version: 1\n";
        let expected = 1;
        let (_, actual) = version(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_name() {
        let input = "name: test\n";
        let expected = "test";
        let (_, actual) = name(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_frequency() {
        let input = "frequency: 1000\n";
        let expected = 1000;
        let (_, actual) = frequency(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_duty_cycle() {
        let input = "duty_cycle: 0.5\n";
        let expected = 0.5;
        let (_, actual) = duty_cycle(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_data() {
        let input = "data: 1 2 3 4 5\n";
        let expected = vec![1, 2, 3, 4, 5];
        let (_, actual) = data(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_signal_type() {
        let input = "type: raw\n";
        let expected = SignalType::Raw;
        let (_, actual) = signal_type(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_saved_signal() {
        let input = indoc! {"
            #
            name: test
            type: raw
            frequency: 1000
            duty_cycle: 0.5
            data: 1 2 3 4 5
        "};
        let expected = SavedSignal {
            name: "test".to_string(),
            r#type: SignalType::Raw,
            frequency: 1000,
            duty_cycle: 0.5,
            data: vec![1, 2, 3, 4, 5],
        };
        let (_, actual) = saved_signal(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_dump_file() {
        let input = indoc! {"
            Filetype: IR signals file
            Version: 1
            #
            name: test
            type: raw
            frequency: 1000
            duty_cycle: 0.5
            data: 1 2 3 4 5
        "};
        let expected = DumpFile {
            version: 1,
            signals: vec![SavedSignal {
                name: "test".to_string(),
                r#type: SignalType::Raw,
                frequency: 1000,
                duty_cycle: 0.5,
                data: vec![1, 2, 3, 4, 5],
            }],
        };
        let (_, actual) = dump_file(input).unwrap();
        assert_eq!(expected, actual);
    }
}
