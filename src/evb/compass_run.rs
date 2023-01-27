use std::fs::File;
use std::path::PathBuf;
use std::collections::HashMap;

use tar::Archive;
use flate2::read::GzDecoder;
use polars::prelude::*;
use log::{info};

use super::compass_file::{CompassRunError, CompassFile};
use super::event_builder::EventBuilder;
use super::channel_map::{ChannelMap};
use super::sps_data::{SPSData, SPSDataField};

#[derive(Debug)]
pub struct RunParams {
    pub run_archive_path: PathBuf,
    pub unpack_dir_path: PathBuf,
    pub output_file_path: PathBuf,
    pub chanmap_file_path: PathBuf,
    pub coincidence_window: f64
}

pub fn make_dataframe(data: Vec<SPSData>) -> Result<DataFrame, PolarsError> {

    let fields = SPSDataField::get_field_vec();
    let mut column_map: HashMap<SPSDataField, PrimitiveChunkedBuilder<Float64Type>> = fields.into_iter()
                                                                          .map(|f| -> (SPSDataField, PrimitiveChunkedBuilder<Float64Type>) { 
                                                                               (f.clone(), PrimitiveChunkedBuilder::<Float64Type>::new(&f.get_column_name(), data.len()))
                                                                              })
                                                                          .collect();

    for datum in data {
        datum.fields()
             .into_iter()
             .for_each(|f| { column_map.get_mut(f.0).unwrap().append_value(*f.1) })
    }

    let columns: Vec<Series> = column_map.into_iter().map(|f| -> Series { f.1.finish().into() }).collect();

    DataFrame::new(columns)
}

pub fn process_run(params: &RunParams) -> Result<(), CompassRunError> {
    let archive_file = File::open(&params.run_archive_path)?;
    let mut decompressed_archive = Archive::new(GzDecoder::new(archive_file));
    decompressed_archive.unpack(&params.unpack_dir_path)?;

    let mut files: Vec<CompassFile> = vec![];
    let mut total_count: u64 = 0;    
    for item in params.unpack_dir_path.read_dir()? {
        files.push(CompassFile::new(&item?.path())?);
        files.last_mut().unwrap().set_hit_used();
        files.last_mut().unwrap().get_top_hit()?;
        total_count += files.last().unwrap().get_number_of_hits();
    }

    let mut evb = EventBuilder::new(&params.coincidence_window);
    let channel_map = ChannelMap::new(&params.chanmap_file_path)?;

    let mut earliest_file_index: Option<usize>;
    let mut analyzed_data: Vec<SPSData> = vec![];

    let mut count: u64 = 0;
    let mut flush_count: u64 = 0;
    let flush_percent = 0.1;
    let flush_val: u64 = ((total_count as f64) * flush_percent) as u64;

    loop {

        earliest_file_index = Option::None;
        for i in 0..files.len() {
            if !files[i].is_eof() {
                let hit = files[i].get_top_hit()?;
                if hit.is_default() { continue; }

                earliest_file_index = match earliest_file_index {
                    None => Some(i),
                    Some(index) => {
                        if hit.timestamp < files[index].get_top_hit()?.timestamp {
                            Some(i)
                        } else {
                            Some(index)
                        }
                    }
                };
            }
        }

        match earliest_file_index {
            None => break, //This is how we exit
            Some(i) => {
                let hit = files[i].get_top_hit()?;
                evb.push_hit(hit);
                files[i].set_hit_used();
            }
        }

        if evb.is_event_ready() {
            let data = SPSData::new(evb.get_ready_event(), &channel_map);
            if !data.is_default() {
                analyzed_data.push(data);
            }
        }

        count += 1;
        if count == flush_val {
            flush_count += 1;
            count = 0;
            info!("Percent of data processed: {}%", (flush_count as f64 * flush_percent * 100.0) as i32);
        }
    }

    let mut df = make_dataframe(analyzed_data)?;
    let mut output_file = File::create(&params.output_file_path)?;
    ParquetWriter::new(&mut output_file).finish(&mut df)?;
    
    return Ok(());
}