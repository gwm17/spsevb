use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum MassError {
    MassNotFoundError,
    MassFileNotFoundError
}

#[derive(Debug, Clone)]
struct NuclearData {
    z: u32,
    a: u32,
    mass: f64,
    isotope: String,
    element: String
}



#[derive(Debug, Clone)]
pub struct MassMap {
    map: HashMap<u32, NuclearData>,
    file: PathBuf
}

impl Default for MassMap {
    fn default() -> Self {
        MassMap { map: HashMap::new(), file: PathBuf::new() }
    }
}

#[derive(Debug, Clone)]
pub struct KineParameters {
    zt: u32,
    at: u32,
    zp: u32,
    ap: u32,
    ze: u32,
    ae: u32,
    b_field: f64, //kG
    sps_angle: f64, //deg
    projectile_ke: f64, //MeV
}

impl Default for KineParameters {
    fn default() -> Self {
        KineParameters {
            zt: 1,
            at: 1,
            zp: 1,
            ap: 1,
            ze: 1,
            ae: 1,
            b_field: 7.9,
            sps_angle: 0.0,
            projectile_ke: 16.0,
        }
    }
}

impl KineParameters {
    pub fn new() -> Self {
        KineParameters::default()
    }
}

fn calculate_weights_z_offset(params: &KineParameters) -> f64 {
    0.0
}

pub fn calculate_weights(params: &KineParameters) -> (f64, f64) {
    (0.0, 0.0)
}
