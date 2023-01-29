
#[macro_use]
extern crate bitflags;
extern crate rand;
extern crate tar;
extern crate flate2;
extern crate nom;
extern crate polars;
extern crate log;
extern crate simplelog;
extern crate eframe;

mod evb;
mod ui;

use std::path::PathBuf;
use log::{info, error};
use crate::evb::compass_run::{RunParams, process_run};
use crate::ui::app::EVBApp;

//Temporary
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
fn test_no_gui() {
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

fn main() {
    simplelog::TermLogger::init(simplelog::LevelFilter::Info, 
                                simplelog::Config::default(),
                                simplelog::TerminalMode::Mixed, 
                                simplelog::ColorChoice::Auto)
                            .unwrap();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("SPS Event Builder", native_options, Box::new(|cc| Box::new( EVBApp::new(cc) )));
}
