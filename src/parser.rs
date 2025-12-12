use nom::IResult;
use nom::Parser;
use nom::character::complete::{char, u8, u16};
use nom::sequence::{delimited, preceded};

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::event::CombatEvent;
use crate::event::Timestamp;

pub fn parse_log_file<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<CombatEvent>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut events = Vec::new();

    // Read as bytes and convert from ISO-8859-1 (Latin-1) to UTF-8
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    let content: String = bytes.iter().map(|&b| b as char).collect();

    for (line_number, line) in content.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        if let Some(event) = parse_line(line_number + 1, line) {
            events.push(event);
        }
    }

    Ok(events)
}

fn parse_line(line_number: usize, _line: &str) -> Option<CombatEvent> {
    let (_remaining, ts) = parse_timestamp(_line).ok()?;
    let event = CombatEvent {
        line_number,
        timestamp: ts,
        ..Default::default()
    };

    Some(event)
}

pub fn parse_timestamp(input: &str) -> IResult<&str, Timestamp> {
    let (input, (hour, minute, second, millis)) = delimited(
        char('['),
        (
            u8,
            preceded(char(':'), u8),
            preceded(char(':'), u8),
            preceded(char('.'), u16),
        ),
        char(']'),
    )
    .parse(input)?;

    Ok((
        input,
        Timestamp {
            hour,
            minute,
            second,
            millis,
        },
    ))
}
