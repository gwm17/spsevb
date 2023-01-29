use eframe::egui;
use std::path;
use std::fs;
use std::env;

#[derive(Debug, Clone)]
enum FDType {
    OpenFile,
    OpenDirectory,
    SaveFile
}

#[derive(Debug)]
pub struct FileDialog {
    is_active: bool,
    dialog_type: FDType,
    selected_item: Option<path::PathBuf>
}

impl Default for FileDialog {
    fn default() -> Self {
        FileDialog { is_active: false, dialog_type: FDType::OpenFile, selected_item: None}
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

    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        match self.dialog_type {
            FDType::OpenFile => self.show_open_file(ctx),
            FDType::OpenDirectory => self.show_open_directory(ctx),
            FDType::SaveFile => self.show_save_file(ctx)
        }
    }

    fn show_open_file(&mut self, ctx: &egui::Context) -> bool {
        let mut selected: bool = false;
        egui::Window::new("Open File").open(&mut self.is_active.clone()).show(ctx, |ui| {
            ui.label("Open that file!");
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
        return selected;
    }

    fn show_open_directory(&mut self, ctx: &egui::Context) -> bool {
        let mut selected: bool = false;
        egui::Window::new("Open Directory").open(&mut self.is_active.clone()).show(ctx, |ui| {
            ui.label("Open that directory!");
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
        return selected;
    }

    fn show_save_file(&mut self, ctx: &egui::Context) -> bool {
        let mut selected: bool = false;
        egui::Window::new("Open Directory").open(&mut self.is_active.clone()).show(ctx, |ui| {
            ui.label("Open that directory!");
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
        return selected;
    }
}