use crate::compass_run::{RunParams, process_run};

#[macro_use]
extern crate bitflags;
extern crate rand;
extern crate tar;
extern crate flate2;
extern crate nom;
extern crate polars;
extern crate log;
extern crate simplelog;

mod compass_data;
mod compass_file;
mod compass_run;
mod event_builder;
mod sps_data;
mod channel_map;

use std::path::PathBuf;
use log::{info, error};

fn main() {
    simplelog::TermLogger::init(simplelog::LevelFilter::Trace, 
                                simplelog::Config::default(),
                                simplelog::TerminalMode::Mixed, 
                                simplelog::ColorChoice::Auto)
                            .unwrap();
    println!("Hello, world!");

    let params = RunParams { 
        run_archive_path: PathBuf::from("/media/data/gwm17/evb_testspace/raw_binary/run_1.tar.gz"),
        unpack_dir_path: PathBuf::from("/media/data/gwm17/evb_testspace/temp_binary/"),
        output_file_path: PathBuf::from("/media/data/gwm17/evb_testspace/rust/run_1.parquet"),
        chanmap_file_path: PathBuf::from("./etc/ChannelMap.txt"),
        coincidence_window: 3.0e3
    };

    info!("Run parameters: {:?}", params);
    match process_run(&params) {
        Err(x) => error!("An error occured when running process_run! Here's the error info {:?}", x),
        Ok(()) => info!("Successfully proccessed run!")
    }
}
