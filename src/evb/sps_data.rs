use super::compass_data::CompassData;
use super::channel_map::{ChannelMap, SPSChannelType};

use std::collections::HashMap;
use std::hash::Hash;

use strum::IntoEnumIterator;
use strum_macros::{EnumIter, AsRefStr};

const INVALID_VALUE: f64 = -1.0e6;

#[derive(Debug, Clone, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum SPSDataField {
    AnodeFrontEnergy,
    AnodeFrontShort,
    AnodeFrontTime,
    AnodeBackEnergy,
    AnodeBackShort,
    AnodeBackTime,
    ScintLeftEnergy,
    ScintLeftShort,
    ScintLeftTime,
    ScintRightEnergy,
    ScintRightShort,
    ScintRightTime,
    CathodeEnergy,
    CathodeShort,
    CathodeTime,
    DelayFrontLeftEnergy,
    DelayFrontLeftShort,
    DelayFrontLeftTime,
    DelayFrontRightEnergy,
    DelayFrontRightShort,
    DelayFrontRightTime,
    DelayBackLeftEnergy,
    DelayBackLeftShort,
    DelayBackLeftTime,
    DelayBackRightEnergy,
    DelayBackRightShort,
    DelayBackRightTime,
    X1,
    X2,
    Xavg,
    Theta
}

impl SPSDataField {
    //Returns a list of fields for iterating over
    pub fn get_field_vec() -> Vec<SPSDataField> {
        SPSDataField::iter().collect()
    }
}

#[derive(Debug, Clone)]
pub struct SPSData {
    pub fields: HashMap<SPSDataField, f64>
}

impl Default for SPSData {
    fn default() -> Self {
        let fields = SPSDataField::get_field_vec();
        let mut data = SPSData { fields: HashMap::new() };
        fields.into_iter().for_each(|f| { data.fields.insert(f, INVALID_VALUE); });
        return data;
    }
}

impl SPSData {

    pub fn new(event: Vec<CompassData>, map: &ChannelMap, weights: Option<(f64, f64)>) -> SPSData {
        let mut data = SPSData::default();

        let mut dfl_time = INVALID_VALUE;
        let mut dfr_time = INVALID_VALUE;
        let mut dbl_time = INVALID_VALUE;
        let mut dbr_time = INVALID_VALUE;
        
        for hit in event.iter() {
            //Fill out detector fields using channel map
            match map.get_data_field(&hit.uuid) {
                SPSChannelType::ScintLeft => {
                    data.fields.insert(SPSDataField::ScintLeftEnergy, hit.energy);
                    data.fields.insert(SPSDataField::ScintLeftShort, hit.energy_short);
                    data.fields.insert(SPSDataField::ScintLeftTime, hit.timestamp);
                }

                SPSChannelType::ScintRight => {
                    data.fields.insert(SPSDataField::ScintRightEnergy, hit.energy);
                    data.fields.insert(SPSDataField::ScintRightShort, hit.energy_short);
                    data.fields.insert(SPSDataField::ScintRightTime, hit.timestamp);
                }

                SPSChannelType::Cathode => {
                    data.fields.insert(SPSDataField::CathodeEnergy, hit.energy);
                    data.fields.insert(SPSDataField::CathodeShort, hit.energy_short);
                    data.fields.insert(SPSDataField::CathodeTime, hit.timestamp);
                }

                SPSChannelType::DelayFrontRight => {
                    data.fields.insert(SPSDataField::DelayFrontRightEnergy, hit.energy);
                    data.fields.insert(SPSDataField::DelayFrontRightShort, hit.energy_short);
                    data.fields.insert(SPSDataField::DelayFrontRightTime, hit.timestamp);
                    dfr_time = hit.timestamp;
                }

                SPSChannelType::DelayFrontLeft => {
                    data.fields.insert(SPSDataField::DelayFrontLeftEnergy, hit.energy);
                    data.fields.insert(SPSDataField::DelayFrontLeftShort, hit.energy_short);
                    data.fields.insert(SPSDataField::DelayFrontLeftTime, hit.timestamp);
                    dfl_time = hit.timestamp;
                }

                SPSChannelType::DelayBackRight => {
                    data.fields.insert(SPSDataField::DelayBackRightEnergy, hit.energy);
                    data.fields.insert(SPSDataField::DelayBackRightShort, hit.energy_short);
                    data.fields.insert(SPSDataField::DelayBackRightTime, hit.timestamp);
                    dbr_time = hit.timestamp;
                }

                SPSChannelType::DelayBackLeft => {
                    data.fields.insert(SPSDataField::DelayBackLeftEnergy, hit.energy);
                    data.fields.insert(SPSDataField::DelayBackLeftShort, hit.energy_short);
                    data.fields.insert(SPSDataField::DelayBackLeftTime, hit.timestamp);
                    dbl_time = hit.timestamp;
                }

                SPSChannelType::AnodeFront => {
                    data.fields.insert(SPSDataField::AnodeFrontEnergy, hit.energy);
                    data.fields.insert(SPSDataField::AnodeFrontShort, hit.energy_short);
                    data.fields.insert(SPSDataField::AnodeFrontTime, hit.timestamp);
                }

                SPSChannelType::AnodeBack => {
                    data.fields.insert(SPSDataField::AnodeBackEnergy, hit.energy);
                    data.fields.insert(SPSDataField::AnodeBackShort, hit.energy_short);
                    data.fields.insert(SPSDataField::AnodeBackTime, hit.timestamp);
                }

                _ =>  continue
            }
        }

        //Physics
        let mut x1 = INVALID_VALUE;
        let mut x2 = INVALID_VALUE;
        if dfr_time != INVALID_VALUE && dfl_time != INVALID_VALUE {
            x1 = (dfl_time - dfr_time) * 0.5 * 1.0/2.1;
            data.fields.insert(SPSDataField::X1, x1);
        }
        if dbr_time != INVALID_VALUE && dbl_time != INVALID_VALUE {
            x2 = (dbl_time - dbr_time) * 0.5 * 1.0/1.98;
            data.fields.insert(SPSDataField::X2, x2);
        }
        if x1 != INVALID_VALUE && x2 != INVALID_VALUE {
            let diff = x1 -x2;
            if diff > 0.0 {
                data.fields.insert(SPSDataField::Theta, (diff/36.0).atan());
            } else if diff < 0.0 {
                data.fields.insert(SPSDataField::Theta, std::f64::consts::PI + (diff/36.0).atan());
            } else {
                data.fields.insert(SPSDataField::Theta, std::f64::consts::PI * 0.5);
            }

            match weights {
               Some(w) => data.fields.insert(SPSDataField::Xavg, w.0 * x1 + w.1 * x2),
               None => data.fields.insert(SPSDataField::Xavg, INVALID_VALUE)
            };
        }

        return data;
    }

    pub fn is_default(&self) -> bool {
        for f in &self.fields {
            if *f.1 != INVALID_VALUE {
                return false;
            }
        }
        return true;
    }
    
    pub fn fields(&self) -> &HashMap<SPSDataField, f64> {
        &self.fields
    }
}