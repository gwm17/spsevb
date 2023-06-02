
use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::num::ParseIntError;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter};

use super::compass_data::generate_board_channel_uuid;

const INVALID_LOCAL_DET_ID: i32 = -1;
const INVALID_LOCAL_CHANNEL: i32 = -1;

//Channels to be mapped in the ChannelMap, each variant is the verbatim keyword in the channel map
#[derive(Debug, Clone, PartialEq, AsRefStr, EnumIter)]
pub enum SPSChannelType {
    //Detector fields -> can be channel mapped
    AnodeFront,
    AnodeBack,
    ScintLeft,
    ScintRight,
    Cathode,
    DelayFrontLeft,
    DelayFrontRight,
    DelayBackLeft,
    DelayBackRight,
    SabreRing,
    SabreWedge,
    //Invalid channel
    None
}

impl SPSChannelType {
    pub fn get_channel_vec() -> Vec<SPSChannelType> {
        SPSChannelType::iter().collect()
    }
}


#[derive(Debug)]
pub enum ChannelMapError {
    IOError(std::io::Error),
    ParseError(ParseIntError),
    UnidentifiedChannelError
}

impl From<std::io::Error> for ChannelMapError {
    fn from(e: std::io::Error) -> Self {
        return ChannelMapError::IOError(e);
    }
}

impl From<ParseIntError> for ChannelMapError {
    fn from(e: ParseIntError) -> Self {
        return ChannelMapError::ParseError(e);
    }
}

impl std::fmt::Display for ChannelMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelMapError::IOError(x) => write!(f, "Channel map had an error with the input file: {}", x),
            ChannelMapError::ParseError(x) => write!(f, "Channel map had an error parsing the channel map file: {}", x),
            ChannelMapError::UnidentifiedChannelError => write!(f, "Channel map found an unidentified field in the channel map file")
        }
    }
}

impl std::error::Error for ChannelMapError {

}

#[derive(Debug, Clone)]
pub struct ChannelData {
    pub channel_type: SPSChannelType,
    pub local_channel: i32,
    pub local_det_id: i32
}

impl Default for ChannelData {
    fn default() -> Self {
        ChannelData { channel_type: SPSChannelType::None, local_channel: INVALID_LOCAL_CHANNEL, local_det_id: INVALID_LOCAL_DET_ID }
    }
}

#[derive(Debug)]
pub struct ChannelMap {
    map: HashMap<u32, ChannelData>
}

impl ChannelMap {
    pub fn new(file: &Path) -> Result<ChannelMap, ChannelMapError> {
        let mut cmap = ChannelMap { map: HashMap::new() };
        let channel_types = SPSChannelType::get_channel_vec();

        let mut file_handle = File::open(file)?;
        let mut file_contents = String::new();
        file_handle.read_to_string(&mut file_contents)?;

        let mut found_flag;
        for line in file_contents.lines() {
            let entries: Vec<&str> = line.split_whitespace().collect();
            if entries.len() < 3 {
                continue;
            }
            let board: u32 = entries[0].parse()?;
            let channel: u32 = entries[1].parse()?;
            let component = entries[2];

            let mut data = ChannelData::default();
            if entries.len() == 5 {
                data.local_channel = entries[3].parse()?;
                data.local_det_id = entries[4].parse()?;
            }

            found_flag = false;

            for channel_type in &channel_types {
                if component == channel_type.as_ref() {
                    data.channel_type = channel_type.clone();
                    cmap.map.insert(generate_board_channel_uuid(&board, &channel), data);
                    found_flag = true;
                    break;
                }
            }

            if !found_flag {
                return Err(ChannelMapError::UnidentifiedChannelError);
            }
        }
        return Ok(cmap);
    }

    pub fn get_channel_data(&self, uuid: &u32) -> Option<&ChannelData> {
        return self.map.get(uuid);
    }
}