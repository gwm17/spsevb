use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, BufRead};
use std::num::ParseIntError;
use std::num::ParseFloatError;

use super::compass_data::generate_board_channel_uuid;

#[derive(Debug)]
pub enum ShiftError {
    FileError(std::io::Error),
    ChannelError(ParseIntError),
    TimeshiftError(ParseFloatError)
}

impl Display for ShiftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiftError::FileError(x) => write!(f, "ShiftMap had an IO error: {}", x),
            ShiftError::ChannelError(x) => write!(f, "ShiftMap could not parse board/channel: {}", x),
            ShiftError::TimeshiftError(x) => write!(f, "ShiftMap could not parse timeshift: {}", x)
        }
    }
}

impl From<std::io::Error> for ShiftError {
    fn from(value: std::io::Error) -> Self {
        ShiftError::FileError(value)
    }
}

impl From<ParseIntError> for ShiftError {
    fn from(value: ParseIntError) -> Self {
        ShiftError::ChannelError(value)
    }
}

impl From<ParseFloatError> for ShiftError {
    fn from(value: ParseFloatError) -> Self {
        ShiftError::TimeshiftError(value)
    }
}

impl std::error::Error for ShiftError {

}

#[derive(Debug, Clone)]
pub struct ShiftMap {
    map: HashMap<u32, f64>
}

impl ShiftMap {
    pub fn new(path: &Path) -> Result<ShiftMap, ShiftError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut junk = String::new();
        let mut mapper = ShiftMap {
            map: HashMap::new()
        };

        reader.read_line(&mut junk)?;
        for line in reader.lines() {
            match line {
                Ok(line_str) => {
                    let entries: Vec<&str> = line_str.split_whitespace().collect();
                    let board: u32 = entries[0].parse()?;
                    let channel: u32 = entries[1].parse()?;
                    let id = generate_board_channel_uuid(&board, &channel);
                    let shift: f64 = entries[2].parse()?;
                    mapper.map.insert(id, shift);
                },
                Err(x) => return Err(ShiftError::from(x))
            };
        }
        return Ok(mapper);
    }

    pub fn get_timeshift(&self, id: &u32) -> f64 {
        if let Some(value) = self.map.get(id) {
            return *value;
        } else {
            return 0.0;
        }
    }
}