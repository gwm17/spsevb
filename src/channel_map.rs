
use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::num::ParseIntError;

use crate::compass_data::generate_board_channel_uuid;

//Define channel map keywords associated with memory locations
//Must be a keyword associated with EACH SPSChannelType that can be present in a channel map
//Non channel map fields (i.e. physics values) do not need a keyword, only a column name
const ANODE_FRONT_KEYWORD: &str = "ANODEFRONT";
const ANODE_BACK_KEYWORD: &str = "ANODEBACK";
const SCINT_LEFT_KEYWORD: &str = "SCINTLEFT";
const SCINT_RIGHT_KEYWORD: &str = "SCINTRIGHT";
const DELAY_FRONT_LEFT_KEYWORD: &str = "DELAYFRONTLEFT";
const DELAY_FRONT_RIGHT_KEYWORD: &str = "DELAYFRONTRIGHT";
const DELAY_BACK_LEFT_KEYWORD: &str = "DELAYBACKLEFT";
const DELAY_BACK_RIGHT_KEYWORD: &str = "DELAYBACKRIGHT";
const CATHODE_KEYWORD: &str  = "CATHODE";
const NONE_CHANNEL_STRING: &str =  "InvalidChannel";

//Channels to be mapped in the ChannelMap
#[derive(Debug, Clone, PartialEq)]
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
    //Invalid channel
    None
}

impl SPSChannelType {
    pub fn get_keyword(&self) -> &str {
        match self {
            SPSChannelType::AnodeFront => ANODE_FRONT_KEYWORD,
            SPSChannelType::AnodeBack => ANODE_BACK_KEYWORD,
            SPSChannelType::ScintLeft => SCINT_LEFT_KEYWORD,
            SPSChannelType::ScintRight => SCINT_RIGHT_KEYWORD,
            SPSChannelType::Cathode => CATHODE_KEYWORD,
            SPSChannelType::DelayFrontLeft => DELAY_FRONT_LEFT_KEYWORD,
            SPSChannelType::DelayFrontRight => DELAY_FRONT_RIGHT_KEYWORD,
            SPSChannelType::DelayBackLeft => DELAY_BACK_LEFT_KEYWORD,
            SPSChannelType::DelayBackRight => DELAY_BACK_RIGHT_KEYWORD,
            _ => NONE_CHANNEL_STRING
        }
    }

    pub fn get_channel_vec() -> Vec<SPSChannelType> {
        vec![
            SPSChannelType::AnodeFront,
            SPSChannelType::AnodeBack,
            SPSChannelType::ScintLeft,
            SPSChannelType::ScintRight,
            SPSChannelType::Cathode,
            SPSChannelType::DelayFrontLeft,
            SPSChannelType::DelayFrontRight,
            SPSChannelType::DelayBackLeft,
            SPSChannelType::DelayBackRight
        ]
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

pub struct ChannelMap {
    map: HashMap<u32, SPSChannelType>
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
            let entries: Vec<&str> = line.split(" ").collect();
            if entries.len() != 4 {
                continue;
            }

            let board: u32 = entries[0].parse()?;
            let channel: u32 = entries[1].parse()?;
            let component = entries[3];

            found_flag = false;

            for channel_type in &channel_types {
                if component == channel_type.get_keyword() {
                    cmap.map.insert(generate_board_channel_uuid(&board, &channel), channel_type.clone());
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

    pub fn get_data_field(&self, uuid: &u32) -> SPSChannelType {
        match self.map.get(uuid) {
            Some(field) => return field.clone(),
            None => return SPSChannelType::None
        }
    }
}