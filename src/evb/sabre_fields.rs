use strum::IntoEnumIterator;
use strum_macros::{EnumIter, AsRefStr};
use super::used_size::UsedSize;

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

impl UsedSize for SabreField {
    fn get_used_size(&self) -> usize {
        std::mem::size_of::<SabreField>()
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

impl UsedSize for SabreData {
    fn get_used_size(&self) -> usize {
        self.energies.get_used_size()+
        self.times.get_used_size() +
        self.channels.get_used_size() + 
        self.det_ids.get_used_size()
    }
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