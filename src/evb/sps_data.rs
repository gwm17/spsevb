
use super::compass_data::{CompassData};
use super::channel_map::{ChannelMap, SPSChannelType};

use std::collections::HashMap;
use std::hash::Hash;

const INVALID_VALUE: f64 = -1.0e6;

const ANODE_FRONT_COLUMN: &str = "AnodeFront";
const ANODE_BACK_COLUMN: &str = "AnodeBack";
const SCINT_LEFT_COLUMN: &str = "ScintLeft";
const SCINT_RIGHT_COLUMN: &str = "ScintRight";
const DELAY_FRONT_LEFT_COLUMN: &str = "DelayFrontLeft";
const DELAY_FRONT_RIGHT_COLUMN: &str = "DelayFrontRight";
const DELAY_BACK_LEFT_COLUMN: &str = "DelayBackLeft";
const DELAY_BACK_RIGHT_COLUMN: &str = "DelayBackRight";
const CATHODE_COLUMN: &str  = "Cathode";
const X1_COLUMN: &str = "X1";
const X2_COLUMN: &str = "X2";
const XAVG_COLUMN: &str = "Xavg";
const THETA_COLUMN: &str = "Theta";

const ENERGY_MOD: &str = "ENERGY";
const SHORT_MOD: &str = "SHORT";
const TIME_MOD: &str = "TIME";

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
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
    //Returns a String with the name for the associated polars df column
    pub fn get_column_name(&self) -> String {
        match self {
            SPSDataField::AnodeFrontEnergy      => format!("{}{}", ANODE_FRONT_COLUMN, ENERGY_MOD),
            SPSDataField::AnodeFrontShort       => format!("{}{}", ANODE_FRONT_COLUMN, SHORT_MOD),
            SPSDataField::AnodeFrontTime        => format!("{}{}", ANODE_FRONT_COLUMN, TIME_MOD),
            SPSDataField::AnodeBackEnergy       => format!("{}{}", ANODE_BACK_COLUMN, ENERGY_MOD),
            SPSDataField::AnodeBackShort        => format!("{}{}", ANODE_BACK_COLUMN, SHORT_MOD),
            SPSDataField::AnodeBackTime         => format!("{}{}", ANODE_BACK_COLUMN, TIME_MOD),
            SPSDataField::ScintLeftEnergy       => format!("{}{}", SCINT_LEFT_COLUMN, ENERGY_MOD),
            SPSDataField::ScintLeftShort        => format!("{}{}", SCINT_LEFT_COLUMN, SHORT_MOD),
            SPSDataField::ScintLeftTime         => format!("{}{}", SCINT_LEFT_COLUMN, TIME_MOD),
            SPSDataField::ScintRightEnergy      => format!("{}{}", SCINT_RIGHT_COLUMN, ENERGY_MOD),
            SPSDataField::ScintRightShort       => format!("{}{}", SCINT_RIGHT_COLUMN, SHORT_MOD),
            SPSDataField::ScintRightTime        => format!("{}{}", SCINT_RIGHT_COLUMN, TIME_MOD),
            SPSDataField::CathodeEnergy         => format!("{}{}", CATHODE_COLUMN, ENERGY_MOD),
            SPSDataField::CathodeShort          => format!("{}{}", CATHODE_COLUMN, SHORT_MOD),
            SPSDataField::CathodeTime           => format!("{}{}", CATHODE_COLUMN, TIME_MOD),
            SPSDataField::DelayFrontLeftEnergy  => format!("{}{}", DELAY_FRONT_LEFT_COLUMN, ENERGY_MOD),
            SPSDataField::DelayFrontLeftShort   => format!("{}{}", DELAY_FRONT_LEFT_COLUMN, SHORT_MOD),
            SPSDataField::DelayFrontLeftTime    => format!("{}{}", DELAY_FRONT_LEFT_COLUMN, TIME_MOD),
            SPSDataField::DelayFrontRightEnergy => format!("{}{}", DELAY_FRONT_RIGHT_COLUMN, ENERGY_MOD),
            SPSDataField::DelayFrontRightShort  => format!("{}{}", DELAY_FRONT_RIGHT_COLUMN, SHORT_MOD),
            SPSDataField::DelayFrontRightTime   => format!("{}{}", DELAY_FRONT_RIGHT_COLUMN, TIME_MOD),
            SPSDataField::DelayBackLeftEnergy   => format!("{}{}", DELAY_BACK_LEFT_COLUMN, ENERGY_MOD),
            SPSDataField::DelayBackLeftShort    => format!("{}{}", DELAY_BACK_LEFT_COLUMN, SHORT_MOD),
            SPSDataField::DelayBackLeftTime     => format!("{}{}", DELAY_BACK_LEFT_COLUMN, TIME_MOD),
            SPSDataField::DelayBackRightEnergy  => format!("{}{}", DELAY_BACK_RIGHT_COLUMN, ENERGY_MOD),
            SPSDataField::DelayBackRightShort   => format!("{}{}", DELAY_BACK_RIGHT_COLUMN, SHORT_MOD),
            SPSDataField::DelayBackRightTime    => format!("{}{}", DELAY_BACK_RIGHT_COLUMN, TIME_MOD),
            SPSDataField::X1                    => X1_COLUMN.to_string(),
            SPSDataField::X2                    => X2_COLUMN.to_string(),
            SPSDataField::Xavg                  => XAVG_COLUMN.to_string(),
            SPSDataField::Theta                 => THETA_COLUMN.to_string()
        }
    }

    //Returns a list of fields for iterating over
    pub fn get_field_vec() -> Vec<SPSDataField> {
        vec![
            SPSDataField::AnodeFrontEnergy,
            SPSDataField::AnodeFrontShort,
            SPSDataField::AnodeFrontTime,
            SPSDataField::AnodeBackEnergy,
            SPSDataField::AnodeBackShort,
            SPSDataField::AnodeBackTime,
            SPSDataField::ScintLeftEnergy,
            SPSDataField::ScintLeftShort,
            SPSDataField::ScintLeftTime,
            SPSDataField::ScintRightEnergy,
            SPSDataField::ScintRightShort,
            SPSDataField::ScintRightTime,
            SPSDataField::CathodeEnergy,
            SPSDataField::CathodeShort,
            SPSDataField::CathodeTime,
            SPSDataField::DelayFrontLeftEnergy,
            SPSDataField::DelayFrontLeftShort,
            SPSDataField::DelayFrontLeftTime,
            SPSDataField::DelayFrontRightEnergy,
            SPSDataField::DelayFrontRightShort,
            SPSDataField::DelayFrontRightTime,
            SPSDataField::DelayBackLeftEnergy,
            SPSDataField::DelayBackLeftShort,
            SPSDataField::DelayBackLeftTime,
            SPSDataField::DelayBackRightEnergy,
            SPSDataField::DelayBackRightShort,
            SPSDataField::DelayBackRightTime,
            SPSDataField::X1,
            SPSDataField::X2,
            SPSDataField::Xavg,
            SPSDataField::Theta                
        ]
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