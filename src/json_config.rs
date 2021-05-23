use serde::{Deserialize, Serialize};
use std::{error, fs, io};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct SubSec {
    num_bits: u8,
    holes: Vec<u8>,
    is_specified: bool,
    specified_val: u8,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Section {
    name: String,
    num_bytes: u8,
    sub_secs: Vec<SubSec>,
    is_specified: bool,
    specified_val: u64,
}

/// Read configuration json file and return configuration object
/*pub fn read_config(filename: &str) -> Result<Configrror>{
    let file_data = Box::new(fs::read_to_string(filename).unwrap());
    serde_json::from_str(&file_data);
    //Ok(v)
}*/
pub fn read_config(filename: &str) -> Result<Section, Box<dyn error::Error>> {
    let file_data = fs::read_to_string(filename)?;
    serde_json::from_str(&file_data).map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
}

/// Write configuration object to a json file
pub fn save_config(filename: &str, config: &Section) -> Result<(), io::Error> {
    let json_config = serde_json::to_string(config)?;
    fs::write(filename, json_config)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn it_reads_and_writes_json() {
        // tempdir allows for reading and writing in a directory that will
        // be created and automatically deleted when the tempdir destructor
        // is run
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_output.json");
        let file_path_str = file_path.to_str().unwrap();

        let sub_section1 = SubSec {
            num_bits: 8,
            holes: vec![],
            is_specified: true,
            specified_val: 0xFF,
        };

        let sub_section2 = SubSec {
            num_bits: 8,
            holes: vec![1, 2, 3],
            is_specified: false,
            specified_val: 0,
        };

        let section = Section {
            name: String::from("SDO"),
            num_bytes: 2,
            sub_secs: vec![sub_section1, sub_section2],
            is_specified: false,
            specified_val: 0,
        };

        save_config(file_path_str, &section).unwrap();
        let input_values = read_config(file_path_str).unwrap();
        assert_eq!(input_values, section);
    }
}
