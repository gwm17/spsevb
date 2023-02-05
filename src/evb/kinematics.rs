use super::nuclear_data::{MassMap};

#[derive(Debug, Clone)]
pub struct KineParameters {
    pub target_z: u32,
    pub target_a: u32,
    pub projectile_z: u32,
    pub projectile_a: u32,
    pub ejectile_z: u32,
    pub ejectile_a: u32,
    pub b_field: f64, //kG
    pub sps_angle: f64, //deg
    pub projectile_ke: f64, //MeV
}

impl Default for KineParameters {
    fn default() -> Self {
        KineParameters {
            target_z: 0,
            target_a: 0,
            projectile_z: 0,
            projectile_a: 0,
            ejectile_z: 0,
            ejectile_a: 0,
            b_field: 7.9,
            sps_angle: 0.0,
            projectile_ke: 16.0,
        }
    }
}

impl KineParameters {
    pub fn get_residual_z(&self) -> u32 {
        self.target_z + self.projectile_z - self.ejectile_z
    }

    pub fn get_residual_a(&self) -> u32 {
        self.target_a + self.projectile_a - self.ejectile_a
    }

    pub fn generate_rxn_eqn(&self, nuc_map: &MassMap) -> String {
        let targ_str = match nuc_map.get_data(&self.target_z, &self.target_a) {
            Some(data) => &data.isotope,
            None => "Invalid"
        };

        let proj_str = match nuc_map.get_data(&self.projectile_z, &self.projectile_a) {
            Some(data) => &data.isotope,
            None => "Invalid"
        };

        let eject_str = match nuc_map.get_data(&self.ejectile_z, &self.ejectile_a) {
            Some(data) => &data.isotope,
            None => "Invalid"
        };

        let resid_str = match nuc_map.get_data(&self.get_residual_z(), &self.get_residual_a()) {
            Some(data) => &data.isotope,
            None => "Invalid"
        };

        format!("{}({},{}){}", targ_str, proj_str, eject_str, resid_str)
    }
}

fn calculate_z_offset(params: &KineParameters, nuc_map: &MassMap) -> Option<f64> {
    Some(0.0)
}

pub fn calculate_weights(params: &KineParameters, nuc_map: &MassMap) -> (f64, f64) {
    (0.0, 0.0)
}
