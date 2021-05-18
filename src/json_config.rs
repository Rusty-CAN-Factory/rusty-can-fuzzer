use serde::{Deserialize, Serialize};
use std::{fs, io};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Config {
    data_type: String,
    name: String,
}

/// Read configuration json file and return configuration object
pub fn read_config(filename: &str) -> Config {
    let file_data = fs::read_to_string(filename).unwrap();
    let v: Config = serde_json::from_str(&file_data).unwrap();
    v
}

/// Write configuration object to a json file
pub fn save_config(filename: &str, config: &Config) -> Result<(), io::Error> {
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

        let config_values = Config {
            data_type: "Type".to_string(),
            name: "name".to_string(),
        }; 
        
        save_config(file_path_str, &config_values).unwrap();
        let input_values = read_config(file_path_str);
        assert_eq!(input_values, config_values);
    }
}