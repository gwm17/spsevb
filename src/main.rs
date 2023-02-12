mod evb;
mod ui;

use crate::ui::app::EVBApp;

fn main() {
    simplelog::TermLogger::init(simplelog::LevelFilter::Info, 
                                simplelog::Config::default(),
                                simplelog::TerminalMode::Mixed, 
                                simplelog::ColorChoice::Auto)
                            .unwrap();
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(eframe::epaint::Vec2 { x: 600.0, y: 430.0 });
    eframe::run_native("SPS Event Builder", native_options, Box::new(|cc| Box::new( EVBApp::new(cc) )));
}
