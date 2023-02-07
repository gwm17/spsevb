use eframe::egui;
use eframe::egui::{RichText, Color32};
use eframe::App;
use native_dialog;
use log::{error, info};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use std::path::PathBuf;

use crate::evb::compass_run::{process_run, RunParams};
use crate::evb::error::EVBError;
use crate::evb::nuclear_data::MassMap;
use crate::evb::kinematics::KineParameters;
use super::ws::{Workspace, WorkspaceError};

#[derive(Debug, Default)]
pub struct EVBApp {
    progress: Arc<Mutex<f32>>,
    workspace: Option<Workspace>,
    channel_map: Option<PathBuf>,
    coincidence_window: f64,
    run_number: i32,
    kine_params: KineParameters,
    rxn_eqn: String,
    mass_map: Arc<MassMap>,
    thread_handle: Option<JoinHandle<Result<(), EVBError>>>
}

impl EVBApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        EVBApp {
            progress: Arc::new(Mutex::new(0.0)),
            workspace: None,
            channel_map: None,
            coincidence_window: 3.0e3,
            run_number: 0,
            kine_params: KineParameters::default(),
            rxn_eqn: String::from("None"),
            mass_map: Arc::new(MassMap::new().expect("Could not open amdc data, shutting down!")),
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
                coincidence_window: self.coincidence_window,
            };

            match self.progress.lock() {
                Ok(mut x) => *x = 0.0,
                Err(_) => error!("Could not aquire lock at starting processor..."),
            };
            let k_params = self.kine_params.clone();
            let mass_handle = self.mass_map.clone();
            self.thread_handle = Some(std::thread::spawn(|| process_run(params, k_params, mass_handle, prog)));
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

            //Menus
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

            //Files/Workspace
            ui.separator();
            ui.label(RichText::new("Run Information").color(Color32::LIGHT_BLUE).size(18.0));
            egui::Grid::new("RunGrid").show(ui,|ui| {
                ui.label("Workspace: ");
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
                ui.end_row();

                ui.label("Channel Map: ");
                ui.label(match &self.channel_map {
                    Some(real_path) => real_path.as_path().to_str().expect("Cannot display channel map!"),
                    None => "None"
                });
                if ui.button("Open").clicked() {
                    let result = native_dialog::FileDialog::new()
                                 .set_location(&std::env::current_dir().expect("Couldn't access runtime directory"))
                                 .add_filter("Text File", &["txt"])
                                 .show_open_single_file();
                    match result {
                        Ok(path) => match path {
                            Some(real_path) => self.channel_map = Some(real_path),
                            None => ()
                        }
                        Err(_) => error!("File dialog error!")
                    }
                }
                ui.end_row();

                ui.label("Coincidence Window (ns)");
                ui.add(egui::widgets::DragValue::new(&mut self.coincidence_window).speed(100).custom_formatter(|n, _| {
                    format!("{:e}", n)
                }));
                ui.end_row();

                ui.label("Run Number");
                ui.add(egui::widgets::DragValue::new(&mut self.run_number).speed(1));
            });

            //Kinematics elements
            ui.separator();
            ui.label(RichText::new("Kinematics").color(Color32::LIGHT_BLUE).size(18.0));
            egui::Grid::new("KineGrid").show(ui,|ui| {
                ui.label("Target Z     ");
                ui.add(egui::widgets::DragValue::new(&mut self.kine_params.target_z).speed(1));
                ui.label("Target A     ");
                ui.add(egui::widgets::DragValue::new(&mut self.kine_params.target_a).speed(1));
                ui.end_row();

                ui.label("Projectile Z");
                ui.add(egui::widgets::DragValue::new(&mut self.kine_params.projectile_z).speed(1));
                ui.label("Projectile A");
                ui.add(egui::widgets::DragValue::new(&mut self.kine_params.projectile_a).speed(1));
                ui.end_row();

                ui.label("Ejectile Z   ");
                ui.add(egui::widgets::DragValue::new(&mut self.kine_params.ejectile_z).speed(1));
                ui.label("Ejectile A   ");
                ui.add(egui::widgets::DragValue::new(&mut self.kine_params.ejectile_a).speed(1));
                ui.end_row();

                ui.label("Magnetic Field(kG)");
                ui.add(egui::widgets::DragValue::new(&mut self.kine_params.b_field).speed(10.0));
                ui.label("SPS Angle(deg)");
                ui.add(egui::widgets::DragValue::new(&mut self.kine_params.sps_angle).speed(1.0));
                ui.label("Projectile KE(MeV)");
                ui.add(egui::widgets::DragValue::new(&mut self.kine_params.projectile_ke).speed(0.01));
                ui.end_row();

                ui.label("Reaction Equation");
                ui.label(&self.rxn_eqn);
                if ui.button("Set Kinematics").clicked() {
                    self.rxn_eqn = self.kine_params.generate_rxn_eqn(&self.mass_map);
                }
            });

            ui.separator();
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
