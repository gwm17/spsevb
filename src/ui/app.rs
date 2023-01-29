use eframe::egui;
use eframe::App;
use log::{error, info};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use std::path::PathBuf;

use crate::evb::compass_run::{process_run, RunParams};
use crate::evb::error::EVBError;

#[derive(Debug, Default)]
pub struct EVBApp {
    progress: Arc<Mutex<f32>>,
    workspace: String,
    channel_map: String,
    thread_handle: Option<JoinHandle<Result<(), EVBError>>>,
}

impl EVBApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        EVBApp {
            progress: Arc::new(Mutex::new(0.0)),
            workspace: String::from(""),
            channel_map: String::from(""),
            thread_handle: None,
        }
    }

    fn check_and_startup_processing_thread(&mut self) {
        if self.thread_handle.is_none() {
            let prog = self.progress.clone();
            //testing
            let params = RunParams {
                run_archive_path: PathBuf::from("/Volumes/Wyndle/evb_test/raw_binary/run_1.tar.gz"),
                unpack_dir_path: PathBuf::from("/Volumes/Wyndle/evb_test/temp_binary/"),
                output_file_path: PathBuf::from("/Volumes/Wyndle/evb_test/built/run_1.parquet"),
                chanmap_file_path: PathBuf::from("./etc/ChannelMap.txt"),
                coincidence_window: 3.0e3,
            };

            match self.progress.lock() {
                Ok(mut x) => *x = 0.0,
                Err(_) => error!("Could not aquire lock at starting processor..."),
            };
            self.thread_handle = Some(std::thread::spawn(|| process_run(params, prog)));
        }
    }

    fn check_and_shutdown_processing_thread(&mut self) {
        if self.thread_handle.is_some() {
            if self.thread_handle.as_ref().unwrap().is_finished() {
                match self.thread_handle.take().unwrap().join() {
                    Ok(result) => {
                        match result {
                            Ok(_) => info!("Finished processing the run"),
                            Err(x) => error!(
                                "An error occured while processing the run: {x}. Job stopped."
                            ),
                        };
                    }
                    Err(_) => error!("An error occured in joining the processing thread!"),
                };
            }
        }
    }
}

impl App for EVBApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open...").clicked() {
                    info!("You opened!");
                }
                if ui.button("Save...").clicked() {
                    info!("You saved!");
                }
            });
            ui.horizontal(|ui| {
                ui.label("Workspace");
                ui.text_edit_singleline(&mut self.workspace);
            });
            ui.horizontal(|ui| {
                ui.label("Channel Map");
                ui.text_edit_singleline(&mut self.channel_map);
            });

            ui.add(
                egui::widgets::ProgressBar::new(match self.progress.lock() {
                    Ok(x) => *x,
                    Err(_) => 0.0,
                })
                .show_percentage(),
            );

            if ui
                .add_enabled(
                    self.thread_handle.is_none(),
                    egui::widgets::Button::new("Run"),
                )
                .clicked()
            {
                info!("Starting processor...");
                self.check_and_startup_processing_thread();
            } else {
                self.check_and_shutdown_processing_thread();
            }
        });
    }
}
