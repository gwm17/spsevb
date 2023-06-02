use strum::IntoEnumIterator;
use strum_macros::{EnumIter, AsRefStr};

#[derive(Debug, Clone, Hash, Eq, PartialOrd, Ord, PartialEq, EnumIter, AsRefStr)]
pub enum SabreField {
    SabreRing,
    SabreWedge
}

impl SabreField {
    //Returns a list of fields for iterating over
    pub fn get_field_vec() -> Vec<SabreField> {
        SabreField::iter().collect()
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialOrd, Ord, PartialEq, EnumIter, AsRefStr)]
pub enum SabreSubField {
    Energy,
    Time,
    Channel,
    DetID
}

#[derive(Debug, Clone)]
pub struct SabreData {
    pub energies: Vec<f64>,
    pub times: Vec<f64>,
    pub channels: Vec<i32>,
    pub det_ids: Vec<i32>
}

impl SabreData {
    pub fn new() -> SabreData {
        SabreData { energies: vec![], times: vec![], channels: vec![], det_ids: vec![] }
    }

    pub fn push(&mut self, energy: f64, time: f64, channel: i32, det_id: i32) {
        self.energies.push(energy);
        self.times.push(time);
        self.channels.push(channel);
        self.det_ids.push(det_id);
    }

    pub fn len(&self) -> usize {
        return self.energies.len();
    }
}