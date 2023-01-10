use crate::compass_run::{RunParams, process_run};

#[macro_use]
extern crate bitflags;
extern crate rand;
extern crate tar;
extern crate flate2;
extern crate nom;
extern crate polars;
extern crate log;

mod compass_data;
mod compass_file;
mod compass_run;
mod event_builder;
mod sps_data;
mod channel_map;

use std::path::PathBuf;
use log::{info, error};

fn main() {
    println!("Hello, world!");

    let params = RunParams { 
        run_archive_path: PathBuf::from("/testing/this/for/compile.tar.gz"),
        unpack_dir_path: PathBuf::from("/testing/this/for/compile/"),
        output_file_path: PathBuf::from("/testing/this/for/compile.parquet"),
        chanmap_file_path: PathBuf::from("/testing/this/for/compile.txt"),
        coincidence_window: 3.0e3
    };

    match process_run(&params) {
        Err(x) => error!("An error occured when running process_run! Here's the error info {:?}", x),
        Ok(()) => info!("Successfully proccessed run!")
    }
}
