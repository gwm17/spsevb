use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path;

use crate::compass_data::{CompassDataType, RawCompassData, CompassData};
use crate::channel_map::{ChannelMapError};


use flate2::DecompressError;
use polars::error::PolarsError;

use nom::number::complete::*;

const BUFFER_SIZE_HITS: usize = 24000; // Size in Compass hits of the buffer for each binary data file

#[derive(Debug)]
pub enum CompassRunError {
    CompressorError(DecompressError),
    WavesError,
    FileError(std::io::Error),
    EofError,
    ParserError,
    ChannelError(ChannelMapError),
    DataFrameError(PolarsError),
    NoError
}

impl From<std::io::Error> for CompassRunError {
    fn from(err: std::io::Error) -> CompassRunError {
        return CompassRunError::FileError(err)
    }
}

impl From<DecompressError> for CompassRunError {
    fn from(err: DecompressError) -> CompassRunError {
        CompassRunError::CompressorError(err)
    }
}

impl From<ChannelMapError> for CompassRunError {
    fn from(err: ChannelMapError) -> CompassRunError {
        CompassRunError::ChannelError(err)
    }
}

impl From<PolarsError> for CompassRunError {
    fn from(err: PolarsError) -> CompassRunError {
        CompassRunError::DataFrameError(err)
    }
}

fn parse_u16(buffer: &[u8]) -> Result<(&[u8], u16), CompassRunError> {
    match le_u16::<&[u8], nom::error::Error<&[u8]>>(buffer) {
        Err(_x) => Err(CompassRunError::ParserError),
        Ok(x) => Ok(x)
    }
}

fn parse_u32(buffer: &[u8]) -> Result<(&[u8], u32), CompassRunError> {
    match le_u32::<&[u8], nom::error::Error<&[u8]>>(buffer) {
        Err(_x) => Err(CompassRunError::ParserError),
        Ok(x) => Ok(x)
    }
}

fn parse_u64(buffer: &[u8]) -> Result<(&[u8], u64), CompassRunError> {
    match le_u64::<&[u8], nom::error::Error<&[u8]>>(buffer) {
        Err(_x) => Err(CompassRunError::ParserError),
        Ok(x) => Ok(x)
    }
}

pub struct CompassFile {
    file_handle: BufReader<File>,
    size_bytes: u64,
    buffer_size_bytes: usize,
    data_type: CompassDataType,
    data_size_bytes: usize,
    current_hit: CompassData,
    is_used: bool,
    is_eof: bool
}

impl CompassFile {
    pub fn new(path: &path::PathBuf) -> Result<CompassFile, CompassRunError> {
        let mut file: File = File::open(path)?;
        let total_size = file.metadata()?.len();

        let mut header:[u8; 2] = [0; 2];
        file.read_exact(&mut header)?;
        let header_word = u16::from_le_bytes(header);

        let mut datatype = CompassDataType::NONE;
        let mut datasize: usize = 16; //minimum 16 bytes for board, channel, timestamp, flags

        if header_word & CompassDataType::ENERGY.bits() != 0 {
            datatype |= CompassDataType::ENERGY;
            datasize += 2;
        }
        if header_word & CompassDataType::ENERGY_SHORT.bits() != 0 {
            datatype |= CompassDataType::ENERGY_SHORT;
            datasize += 2;
        }
        if header_word & CompassDataType::ENERGY_CALIBRATED.bits() != 0 {
            datatype |= CompassDataType::ENERGY_CALIBRATED;
            datasize += 8;
        }
        if header_word & CompassDataType::WAVES.bits() != 0 {
            return Err(CompassRunError::WavesError);
        }
        

        return Ok(CompassFile {
            file_handle: BufReader::with_capacity(datasize * BUFFER_SIZE_HITS, file),
            size_bytes: total_size,
            buffer_size_bytes: datasize * BUFFER_SIZE_HITS,
            data_type: datatype,
            data_size_bytes: datasize,
            current_hit: CompassData::invalid(),
            is_used: false,
            is_eof: false
        });

    }

    pub fn get_top_hit(&mut self) -> Result<&CompassData, CompassRunError> {
        if self.is_used {
            self.current_hit = match self.parse_top_hit() {
                Err(CompassRunError::FileError(e)) => match e.kind() {
                    std::io::ErrorKind::UnexpectedEof => { self.is_eof = true; CompassData::invalid() },
                    _ => return Err(CompassRunError::FileError(e))
                }
                Ok(data) => data,
                Err(x) => return Err(x)
            }
        }

        return Ok(&self.current_hit);
    }

    fn parse_top_hit(&mut self) -> Result<CompassData, CompassRunError> {

        let mut raw_data = RawCompassData{board: 0, channel: 0, timestamp: 0, energy: 0, energy_calibrated: 0, energy_short: 0};

        let mut dataword: Vec<u8> = vec![0; self.data_size_bytes];
        self.file_handle.read_exact(&mut dataword)?;
        let mut dataslice = dataword.as_slice();

        (dataslice, raw_data.board) = parse_u16(dataslice)?;
        (dataslice, raw_data.channel) = parse_u16(dataslice)?;
        (dataslice, raw_data.timestamp) = parse_u64(dataslice)?;
        if self.data_type.bits() & CompassDataType::ENERGY.bits() != 0 {
            (dataslice, raw_data.energy) = parse_u16(dataslice)?;
        }
        if self.data_type.bits() & CompassDataType::ENERGY_CALIBRATED.bits() != 0 {
            (dataslice, raw_data.energy_calibrated) = parse_u64(dataslice)?;
        }
        if self.data_type.bits() & CompassDataType::ENERGY_SHORT.bits() != 0 {
            (dataslice, raw_data.energy_short) = parse_u16(dataslice)?;
        }
        let (_dataslice, _flags) = parse_u32(dataslice)?;

        Ok(CompassData::new(&raw_data))
    }

    pub fn is_eof(&self) -> bool {
        return self.is_eof;
    }

    pub fn set_hit_used(&mut self) {
        self.is_used = true;
    }
}