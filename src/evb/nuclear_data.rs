use std::collections::HashMap;
use std::path::PathBuf;
use std::io::BufRead;
use std::fmt::Display;
use std::error::Error;

#[derive(Debug)]
pub enum MassError {
    MassFileNotFoundError,
    MassFileParseError,
    MassFileParseIntError(std::num::ParseIntError),
    MassFileParseFloatError(std::num::ParseFloatError)
}

impl From<std::io::Error> for MassError {
    fn from(_value: std::io::Error) -> Self {
        MassError::MassFileNotFoundError
    }
}

impl From<std::num::ParseIntError> for MassError {
    fn from(value: std::num::ParseIntError) -> Self {
        MassError::MassFileParseIntError(value)
    }
}

impl From<std::num::ParseFloatError> for MassError {
    fn from(value: std::num::ParseFloatError) -> Self {
        MassError::MassFileParseFloatError(value)
    }
}

impl Display for MassError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MassError::MassFileNotFoundError => write!(f, "Could not find and open amdc mass file!"),
            MassError::MassFileParseError => write!(f, "Unable to parse amdc mass file!"),
            MassError::MassFileParseIntError(e) => write!(f, "Unable to parse amdc mass file with error {}", e),
            MassError::MassFileParseFloatError(e) => write!(f, "Unable to parse amdc mass file with error {}", e)
        }
    }
}

impl Error for MassError {

}

#[derive(Debug, Clone)]
pub struct NuclearData {
    pub z: u32,
    pub a: u32,
    pub mass: f64,
    pub isotope: String,
    pub element: String
}

impl Default for NuclearData {
    fn default() -> Self {
        NuclearData { z: 0, a: 0, mass: 0.0, isotope: String::from("None"), element: String::from("None") }
    }
}

fn generate_nucleus_id(z: &u32, a: &u32) -> u32 {
    if z > a { z * z + z + a } else { a * a + z }
}



#[derive(Debug, Clone, Default)]
pub struct MassMap {
    map: HashMap<u32, NuclearData>,
    file: PathBuf
}

impl MassMap {
    pub fn new() -> Result<Self, MassError> {
        let mut map = MassMap { map: HashMap::new(), file: PathBuf::from(std::env::current_dir()?.join("etc").join("amdc_2016.txt")) };
        map.init()?;
        return Ok(map);
    }

    fn init(&mut self) -> Result<(), MassError> {
        let file = std::fs::File::open(&self.file)?;
        let mut reader = std::io::BufReader::new(file);
        let mut junk = String::new();
        reader.read_line(&mut junk)?;
        reader.read_line(&mut junk)?;
        let lines = reader.lines();
        
        for line in lines {
            match line {
                Ok(line_str) => {
                    let entries: Vec<&str> = line_str.split_whitespace().collect();
                    let mut data = NuclearData::default();
                    data.z = entries[1].parse()?;
                    data.a = entries[2].parse()?;
                    data.element = String::from(entries[3]);
                    data.isotope = format!("{}{}", data.a, data.element);
                    self.map.insert(generate_nucleus_id(&data.z, &data.a), data);
                },
                Err(_) => return Err(MassError::MassFileParseError)
            };
        }

        Ok(())
    }

    pub fn get_data(&self, z: &u32, a: &u32) -> Option<&NuclearData> {
        self.map.get(&generate_nucleus_id(z, a))
    }
}