
#[macro_use]
extern crate bitflags;
extern crate rand;
extern crate tar;
extern crate flate2;
extern crate nom;
extern crate polars;
extern crate log;
extern crate simplelog;

mod evb;

use std::path::PathBuf;
use log::{info, error};
use crate::evb::compass_run::{RunParams, process_run};

//Temporary
use std::sync::{Arc, Mutex};
fn main() {
    simplelog::TermLogger::init(simplelog::LevelFilter::Trace, 
                                simplelog::Config::default(),
                                simplelog::TerminalMode::Mixed, 
                                simplelog::ColorChoice::Auto)
                            .unwrap();
    println!("Hello, world!");

    let params = RunParams { 
        run_archive_path: PathBuf::from("/Volumes/Wyndle/evb_test/raw_binary/run_1.tar.gz"),
        unpack_dir_path: PathBuf::from("/Volumes/Wyndle/evb_test/temp_binary/"),
        output_file_path: PathBuf::from("/Volumes/Wyndle/evb_test/built/run_1.parquet"),
        chanmap_file_path: PathBuf::from("./etc/ChannelMap.txt"),
        coincidence_window: 3.0e3
    };

    let progress = Arc::new(Mutex::new(0.0));
    info!("Run parameters: {:?}", params);
    match process_run(params, progress.clone()) {
        Err(x) => error!("An error occured when running process_run! Here's the error info {:?}", x),
        Ok(()) => info!("Successfully proccessed run!")
    }

    info!("How much did we progress? {}", *progress.lock().unwrap());
}
