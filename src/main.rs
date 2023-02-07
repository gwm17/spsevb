
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
extern crate egui_extras;
extern crate native_dialog;

mod evb;
mod ui;

use crate::ui::app::EVBApp;

//Temporary

fn main() {
    simplelog::TermLogger::init(simplelog::LevelFilter::Info, 
                                simplelog::Config::default(),
                                simplelog::TerminalMode::Mixed, 
                                simplelog::ColorChoice::Auto)
                            .unwrap();
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(eframe::epaint::Vec2 { x: 780.0, y: 420.0 });
    eframe::run_native("SPS Event Builder", native_options, Box::new(|cc| Box::new( EVBApp::new(cc) )));
}
