use std::collections::HashMap;
use std::fs::File;
use std::path::{PathBuf, Path};

use flate2::read::GzDecoder;
use polars::prelude::*;
use std::sync::{Mutex, Arc};
use tar::Archive;

use super::channel_map::ChannelMap;
use super::scaler_list::ScalerList;
use super::compass_file::CompassFile;
use super::event_builder::EventBuilder;
use super::sps_data::{SPSData, SPSDataField};
use super::error::EVBError;
use super::nuclear_data::MassMap;
use super::kinematics::{KineParameters, calculate_weights};

#[derive(Debug)]
struct RunParams {
    pub run_archive_path: PathBuf,
    pub unpack_dir_path: PathBuf,
    pub output_file_path: PathBuf,
    pub scalerlist_file_path: PathBuf,
    pub scalerout_file_path: PathBuf,
    pub coincidence_window: f64,
}

fn clean_up_unpack_dir(unpack_dir: &Path) -> Result<(), EVBError> {

    for item in unpack_dir.read_dir()? {
        if let Ok(entry) = item {
            if entry.metadata()?.is_file() {
                std::fs::remove_file(entry.path())?;
            }
        }
    }

    Ok(())
}

fn make_dataframe(data: Vec<SPSData>) -> Result<DataFrame, PolarsError> {
    let fields = SPSDataField::get_field_vec();
    let mut column_map: HashMap<SPSDataField, PrimitiveChunkedBuilder<Float64Type>> = fields
        .into_iter()
        .map(
            |f| -> (SPSDataField, PrimitiveChunkedBuilder<Float64Type>) {
                (
                    f.clone(),
                    PrimitiveChunkedBuilder::<Float64Type>::new(f.as_ref(), data.len()),
                )
            },
        )
        .collect();

    for datum in data {
        datum
            .fields()
            .into_iter()
            .for_each(|f| column_map.get_mut(f.0).unwrap().append_value(*f.1))
    }

    let columns: Vec<Series> = column_map
        .into_iter()
        .map(|f| -> Series { f.1.finish().into() })
        .collect();

    DataFrame::new(columns)
}

fn process_run(params: RunParams, k_params: &KineParameters, nuc_map: &MassMap, channel_map: &ChannelMap, progress: Arc<Mutex<f32>>) -> Result<(), EVBError> {
    clean_up_unpack_dir(&params.unpack_dir_path)?;

    let archive_file = File::open(&params.run_archive_path)?;
    let mut decompressed_archive = Archive::new(GzDecoder::new(archive_file));
    decompressed_archive.unpack(&params.unpack_dir_path)?;

    let mut scaler_list = ScalerList::new(&params.scalerlist_file_path)?;

    let mut files: Vec<CompassFile> = vec![];
    let mut total_count: u64 = 0;
    for item in params.unpack_dir_path.read_dir()? {
        let filepath = &item?.path();
        if !scaler_list.read_scaler(filepath) {
            files.push(CompassFile::new(filepath)?);
            files.last_mut().unwrap().set_hit_used();
            files.last_mut().unwrap().get_top_hit()?;
            total_count += files.last().unwrap().get_number_of_hits();
        }
    }

    let mut evb = EventBuilder::new(&params.coincidence_window);
    let x_weights = calculate_weights(&k_params, &nuc_map);

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
                if hit.is_default() {
                    continue;
                }

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
            let data = SPSData::new(evb.get_ready_event(), &channel_map, x_weights);
            if !data.is_default() {
                analyzed_data.push(data);
            }
        }

        count += 1;
        if count == flush_val {
            flush_count += 1;
            count = 0;

            match progress.lock() {
                Ok(mut prog) => *prog  = (flush_count as f64 * flush_percent) as f32,
                Err(_) => return Err(EVBError::SyncError)
            };
        }
    }

    let mut df = make_dataframe(analyzed_data)?;
    let mut output_file = File::create(&params.output_file_path)?;
    ParquetWriter::new(&mut output_file).finish(&mut df)?;
    scaler_list.write_scalers(&params.scalerout_file_path)?;

    //To be safe, manually drop all files in unpack dir before deleting all the files
    drop(files);

    clean_up_unpack_dir(&params.unpack_dir_path)?;

    return Ok(());
}

pub struct ProcessParams {
    pub archive_dir: PathBuf,
    pub unpack_dir: PathBuf,
    pub output_dir: PathBuf,
    pub channel_map_filepath: PathBuf,
    pub scaler_list_filepath: PathBuf,
    pub coincidence_window: f64,
    pub run_min: i32,
    pub run_max: i32
}

pub fn process_runs(params: ProcessParams, k_params: KineParameters, progress: Arc<Mutex<f32>>) -> Result<(), EVBError> {
    let channel_map = ChannelMap::new(&params.channel_map_filepath)?;
    let mass_map = MassMap::new()?;
    for run in params.run_min..params.run_max {
        let local_params =  RunParams {
            run_archive_path: params.archive_dir.join(format!("run_{}.tar.gz", run)),
            unpack_dir_path: params.unpack_dir.clone(),
            output_file_path: params.output_dir.join(format!("run_{}.parquet", run)),
            scalerlist_file_path: params.scaler_list_filepath.clone(),
            scalerout_file_path: params.output_dir.join(format!("run_{}_scalers.txt", run)),
            coincidence_window: params.coincidence_window.clone()
        };

        match progress.lock() {
            Ok(mut prog) => *prog  = 0.0,
            Err(_) => return Err(EVBError::SyncError)
        };

        if local_params.run_archive_path.exists() {
            process_run(local_params, &k_params, &mass_map, &channel_map, progress.clone())?;
        }
    }

    Ok(())
}