use flate2::DecompressError;
use polars::error::PolarsError;
use std::error::Error;
use super::channel_map::{ChannelMapError};
use super::nuclear_data::MassError;
use std::fmt::Display;

#[derive(Debug)]
pub enum EVBError {
    CompressorError(DecompressError),
    WavesError,
    FileError(std::io::Error),
    ParserError,
    ChannelError(ChannelMapError),
    DataFrameError(PolarsError),
    MassMapError(MassError),
    SyncError
}

impl From<std::io::Error> for EVBError {
    fn from(err: std::io::Error) -> EVBError {
        return EVBError::FileError(err)
    }
}

impl From<DecompressError> for EVBError {
    fn from(err: DecompressError) -> EVBError {
        EVBError::CompressorError(err)
    }
}

impl From<ChannelMapError> for EVBError {
    fn from(err: ChannelMapError) -> EVBError {
        EVBError::ChannelError(err)
    }
}

impl From<PolarsError> for EVBError {
    fn from(err: PolarsError) -> EVBError {
        EVBError::DataFrameError(err)
    }
}

impl From<MassError> for EVBError {
    fn from(value: MassError) -> Self {
        EVBError::MassMapError(value)
    }
}

impl Display for EVBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EVBError::CompressorError(x) => write!(f, "Run had a decompression error: {}", x),
            EVBError::WavesError => write!(f, "Run found a file with waveform data, which is not supported!"),
            EVBError::FileError(x) => write!(f, "Run had a file I/O error: {}", x),
            EVBError::ParserError => write!(f, "Run had an error parsing the data from files"),
            EVBError::ChannelError(x) => write!(f, "Run had an error occur with the channel map: {}", x),
            EVBError::DataFrameError(x) => write!(f, "Run had an error using polars: {}", x),
            EVBError::MassMapError(x) => write!(f, "Run had an error with the mass data: {}", x),
            EVBError::SyncError => write!(f, "Run was unable to access shared progress resource")
        }
    }
}

impl Error for EVBError {

}