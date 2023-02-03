use eframe::egui;
use eframe::App;
use native_dialog;
use log::{error, info};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use std::path::PathBuf;

use crate::evb::compass_run::{process_run, RunParams};
use crate::evb::error::EVBError;
use super::workspace::{Workspace, WorkspaceError};

#[derive(Debug, Default)]
pub struct EVBApp {
    progress: Arc<Mutex<f32>>,
    workspace: Option<Workspace>,
    channel_map: String,
    run_number: i32,
    thread_handle: Option<JoinHandle<Result<(), EVBError>>>
}

impl EVBApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        EVBApp {
            progress: Arc::new(Mutex::new(0.0)),
            workspace: None,
            channel_map: String::from(""),
            run_number: 0,
            thread_handle: None
        }
    }

    fn check_and_startup_processing_thread(&mut self) -> Result<(), WorkspaceError> {
        if self.thread_handle.is_none() && self.workspace.is_some() {
            let prog = self.progress.clone();
            //testing
            let params = RunParams {
                run_archive_path: self.workspace.as_ref().unwrap().get_raw_binary_file(&self.run_number)?,
                unpack_dir_path: self.workspace.as_ref().unwrap().get_temp_binary_dir()?,
                output_file_path: self.workspace.as_ref().unwrap().get_built_file(&self.run_number)?,
                chanmap_file_path: PathBuf::from("./etc/ChannelMap.txt"),
                coincidence_window: 3.0e3,
            };

            match self.progress.lock() {
                Ok(mut x) => *x = 0.0,
                Err(_) => error!("Could not aquire lock at starting processor..."),
            };
            self.thread_handle = Some(std::thread::spawn(|| process_run(params, prog)));
        }
        Ok(())
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
                if ui.button("Open Config...").clicked() {
                    let result = native_dialog::FileDialog::new()
                               .set_location(&std::env::current_dir().expect("Couldn't access runtime directory"))
                               .add_filter("YAML file", &["yaml"])
                               .show_open_single_file();
                    match result {
                        Ok(path) => match path {
                            Some(real_path) => info!("Selected a path {}", real_path.display()),
                            None => ()
                        }
                        Err(_) => error!("File dialog error!")
                    }
                }
                if ui.button("Save Config...").clicked() {
                    let result = native_dialog::FileDialog::new()
                               .set_location(&std::env::current_dir().expect("Couldn't access runtime directory"))
                               .add_filter("YAML file", &["yaml"])
                               .show_save_single_file();
                    match result {
                        Ok(path) => match path {
                            Some(real_path) => info!("Selected a path {}", real_path.display()),
                            None => ()
                        }
                        Err(_) => error!("File dialog error!")
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Workspace");
                ui.label(match &self.workspace {
                    Some(ws) => ws.get_parent_str(),
                    None => "None"
                });
                if ui.button("Open").clicked() {
                    let result = native_dialog::FileDialog::new()
                                 .set_location(&std::env::current_dir().expect("Couldn't access runtime directory"))
                                 .show_open_single_dir();
                    match result {
                        Ok(path) => match path {
                            Some(real_path) => self.workspace = match Workspace::new(&real_path) {
                                Ok(ws) => Some(ws),
                                Err(e) => {
                                    error!("Error creating workspace: {}", e);
                                    None
                                }
                            },
                            None => ()
                        }
                        Err(_) => error!("File dialog error!")
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Channel Map");
                ui.label(&self.channel_map);
                if ui.button("Open").clicked() {
                    let result = native_dialog::FileDialog::new()
                                 .set_location(&std::env::current_dir().expect("Couldn't access runtime directory"))
                                 .add_filter("Text File", &["txt"])
                                 .show_open_single_file();
                    match result {
                        Ok(path) => match path {
                            Some(real_path) => info!("Selected a path {}", real_path.display()),
                            None => ()
                        }
                        Err(_) => error!("File dialog error!")
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Run Number");
                ui.add(egui::widgets::DragValue::new(&mut self.run_number).speed(1));
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
                match self.check_and_startup_processing_thread() {
                    Ok(_) => (),
                    Err(e) => error!("Could not start processor, recieved the following error: {}", e)
                };
            } else {
                self.check_and_shutdown_processing_thread();
            }
        });
    }
}
