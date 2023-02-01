use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

fn file_type_to_string(ftype: &std::fs::FileType) -> &str {
    if ftype.is_file() {
        "File"
    } else if ftype.is_dir() {
        "Dir"
    } else if ftype.is_symlink() {
        "SymLink"
    } else {
        "UFO"
    }
}

fn file_size_to_human_readable(size: &u64) -> String {
    const PREFIXES: [&str; 7] = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
    let mut size_float = *size as f64;
    let mut prefix_index: usize = 0;
    let step = 1000.0;
    while size_float >= step {
        size_float /= step;
        prefix_index += 1;
    }

    return format!(
        "{}{}",
        size_float as u64,
        match PREFIXES.get(prefix_index) {
            Some(pre) => pre,
            None => "Huge",
        }
    );
}

fn get_file_extension(filename: &OsStr) -> &str {
    std::path::Path::new(filename)
        .extension()
        .unwrap()
        .to_str()
        .unwrap()
}

fn title_string(ftype: &FDType) -> &str {
    match ftype {
        FDType::OpenFile => "Open File",
        FDType::OpenDirectory => "Open Directory",
        FDType::SaveFile => "Save File"
    }
}

#[derive(Debug, Clone)]
enum FDType {
    OpenFile,
    OpenDirectory,
    SaveFile,
}

#[derive(Debug)]
pub struct FileDialog {
    is_active: bool,
    dialog_type: FDType,
    selected_item: String,
    current_directory: PathBuf,
    extension: String,
}

impl Default for FileDialog {
    fn default() -> Self {
        FileDialog {
            is_active: false,
            dialog_type: FDType::OpenFile,
            selected_item: "".into(),
            current_directory: env::current_dir().expect("Cannot access runtime directory?"),
            extension: "yaml".into()
        }
    }
}

impl FileDialog {
    pub fn open_file(&mut self) {
        self.is_active = true;
        self.dialog_type = FDType::OpenFile;
    }

    pub fn open_directory(&mut self) {
        self.is_active = true;
        self.dialog_type = FDType::OpenDirectory;
    }

    pub fn save_file(&mut self) {
        self.is_active = true;
        self.dialog_type = FDType::SaveFile;
    }

    pub fn get_selected_item(&self) -> PathBuf {
        let mut path = self.current_directory.clone();
        path.push(self.selected_item.clone());
        return path;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        match self.dialog_type {
            FDType::OpenFile => self.show_file(ctx),
            FDType::OpenDirectory => self.show_directory(ctx),
            FDType::SaveFile => self.show_file(ctx),
        }
    }

    fn show_file(&mut self, ctx: &egui::Context) -> bool {
        let mut selected: bool = false;
        let mut current_state = self.is_active.clone();
        egui::Window::new(title_string(&self.dialog_type))
            .open(&mut current_state)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Current directory: ");
                    ui.label(
                            self
                            .current_directory
                            .as_path()
                            .to_str()
                            .expect("File dialog could not convert path into string!"),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Extension filter: ");
                    ui.label(&self.extension);
                });

                TableBuilder::new(ui)
                    .striped(true)
                    .columns(Column::initial(200.0).resizable(true), 2)
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Name");
                        });
                        header.col(|ui| {
                            ui.heading("Size");
                        });
                    })
                    .body(|mut body| {
                        for item in fs::read_dir(&self.current_directory.as_path())
                            .expect("File dialog could not access directory")
                        {
                            match item {
                                Ok(entry) => {
                                    body.row(20.0, |mut row| {
                                        let ftype = entry.file_type().unwrap();
                                        let fname = entry.file_name().into_string().unwrap();
                                        row.col(|ui| {
                                            if ui.selectable_label(fname == self.selected_item, format!("{}: {}", file_type_to_string(&ftype), fname))
                                                .clicked()
                                            {
                                                if ftype.is_file() && get_file_extension(&entry.file_name()) == self.extension {
                                                    self.selected_item = fname;
                                                } else if ftype.is_dir() {
                                                    self.current_directory.push(fname);
                                                    self.current_directory = std::fs::canonicalize(
                                                        self.current_directory.clone(),
                                                    )
                                                    .expect("Could not canonicalize path!");
                                                }
                                            }
                                        });
                                        row.col(|ui| {
                                            ui.label(file_size_to_human_readable(
                                                &entry.metadata().unwrap().len(),
                                            ));
                                        });
                                    });
                                }
                                Err(_) => (),
                            }
                        }
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                if ui.selectable_label(false, "..").clicked()
                                {
                                    self.current_directory.push("..");
                                    self.current_directory =
                                        std::fs::canonicalize(self.current_directory.clone())
                                            .expect("Could not canonicalize path!");
                                }
                            });
                            row.col(|ui| {
                                ui.label("N/A");
                            });
                        })
                    });

                ui.horizontal(|ui| {
                    ui.label("Selected Item");
                    ui.text_edit_singleline(&mut self.selected_item);
                });

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        selected = false;
                        self.is_active = false;
                    }
                    if ui.button("Ok").clicked() {
                        selected = true;
                        self.is_active = false;
                    }
                });
            });
        self.is_active &= current_state;
        return selected;
    }

    fn show_directory(&mut self, ctx: &egui::Context) -> bool {
        let mut selected: bool = false;
        let mut current_state = self.is_active.clone();
        egui::Window::new(title_string(&self.dialog_type))
            .open(&mut current_state)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Current directory: ");
                    ui.label(
                            self
                            .current_directory
                            .as_path()
                            .to_str()
                            .expect("File dialog could not convert path into string!"),
                    );
                });

                TableBuilder::new(ui)
                    .striped(true)
                    .columns(Column::initial(200.0).resizable(true), 2)
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Name");
                        });
                        header.col(|ui| {
                            ui.heading("Size");
                        });
                    })
                    .body(|mut body| {
                        for item in fs::read_dir(&self.current_directory.as_path())
                            .expect("File dialog could not access directory")
                        {
                            match item {
                                Ok(entry) => {
                                    body.row(20.0, |mut row| {
                                        let ftype = entry.file_type().unwrap();
                                        let fname = entry.file_name().into_string().unwrap();
                                        row.col(|ui| {
                                            if ui.selectable_label(fname == self.selected_item, format!("{}: {}", file_type_to_string(&ftype), fname))
                                                .clicked()
                                            {
                                                if ftype.is_dir() {
                                                    self.current_directory.push(fname.clone());
                                                    self.current_directory = std::fs::canonicalize(
                                                        self.current_directory.clone(),
                                                    )
                                                    .expect("Could not canonicalize path!");
                                                }
                                            }
                                        });
                                        row.col(|ui| {
                                            ui.label(file_size_to_human_readable(
                                                &entry.metadata().unwrap().len(),
                                            ));
                                        });
                                    });
                                }
                                Err(_) => (),
                            }
                        }
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                if ui.selectable_label(false, "..").clicked()
                                {
                                    self.current_directory.push("..");
                                    self.current_directory =
                                        std::fs::canonicalize(self.current_directory.clone())
                                            .expect("Could not canonicalize path!");
                                }
                            });
                            row.col(|ui| {
                                ui.label("N/A");
                            });
                        })
                    });

                ui.horizontal(|ui| {
                    ui.label("New Directory Name");
                    ui.text_edit_singleline(&mut self.selected_item);
                });

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        selected = false;
                        self.is_active = false;
                    }
                    if ui.button("Ok").clicked() {
                        selected = true;
                        self.is_active = false;
                    }
                });
            });
        self.is_active &= current_state;
        return selected;
    }

}
