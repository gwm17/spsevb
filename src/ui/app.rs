use eframe::egui;
use eframe::egui::{RichText, Color32};
use eframe::App;
use native_dialog;
use log::{error, info};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use serde::{Serialize, Deserialize};

use std::path::{PathBuf, Path};
use std::fs::File;
use std::io::Write;

use crate::evb::compass_run::{process_runs, ProcessParams};
use crate::evb::error::EVBError;
use crate::evb::nuclear_data::MassMap;
use crate::evb::kinematics::KineParameters;
use super::ws::{Workspace, WorkspaceError};

#[derive(Debug, Serialize, Deserialize)]
struct AppParams {
    pub workspace: Option<Workspace>,
    pub channel_map: Option<PathBuf>,
    pub scaler_list: Option<PathBuf>,
    pub shift_map: Option<PathBuf>,
    pub kinematics: KineParameters,
    pub coincidence_window: f64,
    pub run_min: i32,
    pub run_max: i32
}

impl Default for AppParams {
    fn default() -> Self {
        AppParams { workspace: None, channel_map: None, scaler_list: None, shift_map: None, kinematics: KineParameters::default(), coincidence_window: 3.0e3, run_min: 0, run_max: 0 }
    }
}

#[derive(Debug, Default)]
pub struct EVBApp {
    progress: Arc<Mutex<f32>>,

    parameters: AppParams,

    rxn_eqn: String,
    mass_map: MassMap,
    thread_handle: Option<JoinHandle<Result<(), EVBError>>>
}

impl EVBApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        EVBApp {
            progress: Arc::new(Mutex::new(0.0)),
            parameters: AppParams::default(),
            rxn_eqn: String::from("None"),
            mass_map: MassMap::new().expect("Could not open amdc data, shutting down!"),
            thread_handle: None
        }
    }

    fn check_and_startup_processing_thread(&mut self) -> Result<(), WorkspaceError> {
        if self.thread_handle.is_none() && self.parameters.workspace.is_some() 
           && self.parameters.channel_map.is_some() && self.parameters.scaler_list.is_some() {
            let prog = self.progress.clone();
            let r_params = ProcessParams {
                archive_dir: self.parameters.workspace.as_ref().unwrap().get_archive_dir()?,
                unpack_dir: self.parameters.workspace.as_ref().unwrap().get_unpack_dir()?,
                output_dir: self.parameters.workspace.as_ref().unwrap().get_output_dir()?,
                channel_map_filepath: self.parameters.channel_map.as_ref().unwrap().clone(),
                scaler_list_filepath: self.parameters.scaler_list.clone(),
                shift_map_filepath: self.parameters.shift_map.clone(),
                coincidence_window: self.parameters.coincidence_window,
                run_min: self.parameters.run_min,
                run_max: self.parameters.run_max + 1, //Make it [run_min, run_max]
            };

            match self.progress.lock() {
                Ok(mut x) => *x = 0.0,
                Err(_) => error!("Could not aquire lock at starting processor..."),
            };
            let k_params = self.parameters.kinematics.clone();
            self.thread_handle = Some(std::thread::spawn(|| process_runs(r_params, k_params, prog)));
        } else {
            error!("Cannot run event builder without all filepaths specified");
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

    fn write_params_to_file(&self, path: &Path) {
        if let Ok(mut config) = File::create(path) {
            match serde_yaml::to_string(&self.parameters) {
                Ok(yaml_str) => match config.write(yaml_str.as_bytes()){
                    Ok(_) => (),
                    Err(x) => error!("Error writing config to file{}: {}", path.display(), x)
                },
                Err(x) => error!("Unable to write configuration to file, serializer error: {}",x)
            };
        } else {
            error!("Could not open file {} for config write", path.display());
        }
    }

    fn read_params_from_file(&mut self, path: &Path) {
        let yaml_str = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(x) => {
                error!("Unable to open and read config file {} with error {}", path.display(), x);
                return
            }
        };
        
        match serde_yaml::from_str::<AppParams>(&yaml_str) {
            Ok(params) => self.parameters = params,
            Err(x) => error!("Unable to write configuration to file, serializer error: {}",x)
        };
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
                            Some(real_path) => self.read_params_from_file(&real_path),
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
                            Some(real_path) => self.write_params_to_file(&real_path),
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
                ui.label(match &self.parameters.workspace {
                    Some(ws) => ws.get_parent_str(),
                    None => "None"
                });
                if ui.button("Open").clicked() {
                    let result = native_dialog::FileDialog::new()
                                 .set_location(&std::env::current_dir().expect("Couldn't access runtime directory"))
                                 .show_open_single_dir();
                    match result {
                        Ok(path) => match path {
                            Some(real_path) => self.parameters.workspace = match Workspace::new(&real_path) {
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
                ui.label(match &self.parameters.channel_map {
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
                            Some(real_path) => self.parameters.channel_map = Some(real_path),
                            None => ()
                        }
                        Err(_) => error!("File dialog error!")
                    }
                }
                ui.end_row();

                ui.label("Scaler List: ");
                ui.label(match &self.parameters.scaler_list {
                    Some(real_path) => real_path.as_path().to_str().expect("Cannot display scaler list!"),
                    None => "None"
                });
                if ui.button("Open").clicked() {
                    let result = native_dialog::FileDialog::new()
                                 .set_location(&std::env::current_dir().expect("Couldn't access runtime directory"))
                                 .add_filter("Text File", &["txt"])
                                 .show_open_single_file();
                    match result {
                        Ok(path) => match path {
                            Some(real_path) => self.parameters.scaler_list = Some(real_path),
                            None => ()
                        }
                        Err(_) => error!("File dialog error!")
                    }
                }
                ui.end_row();

                ui.label("Shift Map: ");
                ui.label(match &self.parameters.shift_map {
                    Some(real_path) => real_path.as_path().to_str().expect("Cannot display shift map!"),
                    None => "None"
                });
                if ui.button("Open").clicked() {
                    let result = native_dialog::FileDialog::new()
                                 .set_location(&std::env::current_dir().expect("Couldn't access runtime directory"))
                                 .add_filter("Text File", &["txt"])
                                 .show_open_single_file();
                    match result {
                        Ok(path) => match path {
                            Some(real_path) => self.parameters.shift_map = Some(real_path),
                            None => ()
                        }
                        Err(_) => error!("File dialog error!")
                    }
                }
                ui.end_row();

                ui.label("Coincidence Window (ns)");
                ui.add(egui::widgets::DragValue::new(&mut self.parameters.coincidence_window).speed(100).custom_formatter(|n, _| {
                    format!("{:e}", n)
                }));
                ui.end_row();

                ui.label("Run Min");
                ui.add(egui::widgets::DragValue::new(&mut self.parameters.run_min).speed(1));
                ui.end_row();

                ui.label("Run Max");
                ui.add(egui::widgets::DragValue::new(&mut self.parameters.run_max).speed(1));
            });

            //Kinematics elements
            ui.separator();
            ui.label(RichText::new("Kinematics").color(Color32::LIGHT_BLUE).size(18.0));
            egui::Grid::new("KineGrid").show(ui,|ui| {
                ui.label("Target Z     ");
                ui.add(egui::widgets::DragValue::new(&mut self.parameters.kinematics.target_z).speed(1));
                ui.label("Target A     ");
                ui.add(egui::widgets::DragValue::new(&mut self.parameters.kinematics.target_a).speed(1));
                ui.end_row();

                ui.label("Projectile Z");
                ui.add(egui::widgets::DragValue::new(&mut self.parameters.kinematics.projectile_z).speed(1));
                ui.label("Projectile A");
                ui.add(egui::widgets::DragValue::new(&mut self.parameters.kinematics.projectile_a).speed(1));
                ui.end_row();

                ui.label("Ejectile Z   ");
                ui.add(egui::widgets::DragValue::new(&mut self.parameters.kinematics.ejectile_z).speed(1));
                ui.label("Ejectile A   ");
                ui.add(egui::widgets::DragValue::new(&mut self.parameters.kinematics.ejectile_a).speed(1));
                ui.end_row();

                ui.label("Magnetic Field(kG)");
                ui.add(egui::widgets::DragValue::new(&mut self.parameters.kinematics.b_field).speed(10.0));
                ui.label("SPS Angle(deg)");
                ui.add(egui::widgets::DragValue::new(&mut self.parameters.kinematics.sps_angle).speed(1.0));
                ui.label("Projectile KE(MeV)");
                ui.add(egui::widgets::DragValue::new(&mut self.parameters.kinematics.projectile_ke).speed(0.01));
                ui.end_row();

                ui.label("Reaction Equation");
                ui.label(&self.rxn_eqn);
                if ui.button("Set Kinematics").clicked() {
                    self.rxn_eqn = self.parameters.kinematics.generate_rxn_eqn(&self.mass_map);
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
