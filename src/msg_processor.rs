use chrono::Utc;
use core::ops::Range;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use socketcan::*;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::{error, fs, io};

/// SubSection used to define bits within a section definition
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct SubSec {
    name: String,
    num_bits: u8,
    holes: Vec<u8>,
    is_specified: bool,
    specified_val: u8,
}

impl SubSec {
    /// Returns a subsection with the given definition
    pub fn new(
        name: String,
        num_bits: u8,
        holes: Vec<u8>,
        is_specified: bool,
        specified_val: u8,
    ) -> Self {
        Self {
            name,
            num_bits,
            holes,
            is_specified,
            specified_val,
        }
    }

    /// Formatted display of Subsection
    pub fn display(&self) {
        println!(
            "\t\t{}: \n\
                  \t\tnum_bits {}, holes {:?}, \n\
                  \t\tis_specified {}, specified_val {}",
            self.name, self.num_bits, self.holes, self.is_specified, self.specified_val
        );
    }
}

/// Section used to define 1 or more bytes within a message format
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Section {
    name: String,
    num_bytes: u8,
    sub_secs: Vec<SubSec>,
    is_specified: bool,
    specified_val: u64,
}

impl Section {
    /// Returns a section with the given input
    pub fn new(
        name: String,
        num_bytes: u8,
        sub_secs: Vec<SubSec>,
        is_specified: bool,
        specified_val: u64,
    ) -> Self {
        Self {
            name,
            num_bytes,
            sub_secs,
            is_specified,
            specified_val,
        }
    }

    /// Formatted display of a Section
    pub fn display(&self) {
        println!(
            "\t{}: \n\
                  \tnum_bytes {}, is_specified {}, specified_val {}",
            self.name, self.num_bytes, self.is_specified, self.specified_val
        );
    }

    /// formatted display of Section's subsections
    pub fn display_sub_secs(&self) {
        for i in 0..self.sub_secs.len() {
            println!("\t\tSubSec #{}: ", i);
            self.sub_secs[i].display();
        }
    }
}

/// A CAN Message format definition
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct MsgFormat {
    name: String,
    cob_id_range: Range<u32>,
    cob_id_values: Vec<u32>,
    num_sections: u8,
    sections: Vec<Section>,
    is_specified: bool,
    specified_val: u64,
}

impl MsgFormat {
    /// Returns a new message format with the given input
    pub fn new(
        name: String,
        cob_id_range: Range<u32>,
        cob_id_values: Vec<u32>,
        num_sections: u8,
        sections: Vec<Section>,
        is_specified: bool,
        specified_val: u64,
    ) -> Self {
        Self {
            name,
            cob_id_range,
            cob_id_values,
            num_sections,
            sections,
            is_specified,
            specified_val,
        }
    }

    // Formatted display of a message format
    pub fn display(&self) {
        println!(
            "{}: \n\
                  cob_id_range: {:?}, num_sections {}, \n\
                  is_specified {}, specified_val {}",
            self.name, self.cob_id_range, self.num_sections, self.is_specified, self.specified_val
        );
    }

    // Formatted display of sections contained in message format
    pub fn display_sections(&self) {
        for i in 0..self.sections.len() {
            println!("\tSection #{}: ", i);
            self.sections[i].display();
        }
    }
}

/// Generate a random cob_id within message format allowed range or from provided COB-ID list
/// cob_id_values takes precedence over cob_id_range
pub fn random_cob_id_with_format(msg_format: &MsgFormat) -> u32 {
    if !msg_format.cob_id_values.is_empty() {
        return msg_format
            .cob_id_values
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_owned();
    }

    let mut rng = rand::thread_rng();
    rng.gen_range(msg_format.cob_id_range.start..msg_format.cob_id_range.end)
}

/// Generate any random cob_id
/// Uses the valid range 0..2_021
pub fn random_cob_id() -> u32 {
    //total range for cob_id in CANOpen is 0..2_021 (aka 0x0..0x7E5)
    //https://en.wikipedia.org/wiki/CANopen#Predefined_Connection_Set[7]
    let mut rng = rand::thread_rng();
    rng.gen_range(0..2_021)
}

/// Generate any random 8 byte CAN message
pub fn random_msg() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let data: Vec<u8> = (0..8).map(|_| rng.gen_range(0..255)).collect();
    data
}

/// Output provided message data as a can message to a given socket
/// Returns CANFrame that was sent
pub fn create_frame_send_msg(
    cs: &CANSocket,
    channel: &str,
    cob_id: u32,
    data: &[u8],
    rtr: bool,
    err: bool,
) -> Result<CANFrame, Box<dyn Error>> {
    let frame = CANFrame::new(cob_id, data, rtr, err).unwrap();
    cs.write_frame(&frame)?;
    let mut formatted_data = "".to_owned();
    for item in data {
        formatted_data = format!("{}{:02X?} ", formatted_data, item);
    }
    println!(
        "{0:<30} {1:<8} {2:<10} {3:<25}",
        Utc::now().naive_local().format("[%a %b %e %H:%M:%S %Y]:"),
        channel,
        format!("0x{:03X?}", cob_id),
        formatted_data
    );

    Ok(frame)
}

/// Create CAN message data using provided message format
pub fn msg_processor(msg_format: &MsgFormat) -> Vec<u8> {
    let mut sec_result;
    let mut result = 0;
    let msg_byte_array;
    let mut msg_byte_vec: Vec<u8> = Vec::new();
    let mut total_num_bytes = 0;
    if msg_format.sections.len() == 1 {
        result = section_proc(&msg_format.sections[0]);
        total_num_bytes += msg_format.sections[0].num_bytes;
    } else {
        for i in 0..msg_format.sections.len() {
            sec_result = section_proc(&msg_format.sections[i]);
            // shifting the bits to make room for the new result
            result <<= msg_format.sections[i].num_bytes * 8;
            // ORing to add the new result on the end
            result |= sec_result;
            total_num_bytes += msg_format.sections[i].num_bytes;
        }
    }
    // bit shifting the final result so we push the actual
    // code all the way to the left as needed for CAN
    result <<= 64 - total_num_bytes * 8;
    // Chopping up result into a vec<u8>!
    // (done at end because it's simpler to do bit shifting with a single number before now)
    msg_byte_array = result.to_be_bytes();
    for msg in &msg_byte_array {
        msg_byte_vec.push(*msg);
    }
    msg_byte_vec
}

/// Process a given message format section
/// Returns u64 represenation of data generated
fn section_proc(section: &Section) -> u64 {
    let mut sub_sec_result;
    let mut result: u64 = 0;
    if section.is_specified {
        return section.specified_val;
    }

    if section.sub_secs.is_empty() {
        let mut rng = rand::thread_rng();
        let mut max: u64 = 0xFF;
        for _ in 0..section.num_bytes {
            max <<= 8;
            max += 0xFF;
        }
        return rng.gen_range(0..max);
    }

    for i in 0..section.sub_secs.len() {
        sub_sec_result = sub_sec_proc(&section.sub_secs[i]);
        //shifting the bits to make room for the new result
        result <<= section.sub_secs[i].num_bits;
        //ORing to add the new result on the end
        result |= sub_sec_result as u64;
    }
    result
}

/// Process a given message format sub section
/// Returns u8 representation of generated sub section data
pub fn sub_sec_proc(sub_sec: &SubSec) -> u8 {
    let mut rng;
    let range;
    let mut result;
    if sub_sec.is_specified {
        return sub_sec.specified_val;
    }
    rng = rand::thread_rng();
    //u16 instead of u8 because 8 bit long subsecs caused overflow
    range = 2_u16.pow(sub_sec.num_bits as u32) - 1;
    result = rng.gen_range(0..range as u8);
    while sub_sec.holes.contains(&result) {
        result = rng.gen_range(0..range as u8);
    }
    result
}

/// Read configuration files from a given path
/// Path can be a single file or a directory
/// When a directory is provided a recursive search for files
/// will be completed
/// Returns a Vector of all found message formats
pub fn read_configs(path: &Path) -> Result<Vec<MsgFormat>, Box<dyn error::Error>> {
    if !path.is_dir() {
        return Ok(vec![read_config(path)?]);
    }

    let mut result: Vec<MsgFormat> = vec![];
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let filepath = entry.path();
        if filepath.is_dir() {
            result.append(&mut read_configs(&filepath)?);
        } else {
            result.push(read_config(&filepath)?);
        }
    }

    Ok(result)
}

/// Read a single file path into a single message format object
/// Returns a MsgFormat object
fn read_config(filename: &Path) -> Result<MsgFormat, Box<dyn error::Error>> {
    let file_data = fs::read_to_string(filename)?;
    serde_json::from_str(&file_data).map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
}

/// Write configuration object to a json file
pub fn save_config(filename: &str, config: &MsgFormat) -> Result<(), io::Error> {
    let json_config = serde_json::to_string(config)?;
    fs::write(filename, json_config)?;
    Ok(())
}

/// Listen for a message on the can bus and write that message
/// out to a provided log file
/// Assumes that socket read timeout has already been set appropriately
pub fn listen(
    socket: &CANSocket,
    channel: &str,
    logfile: &Path,
    tx_frame: CANFrame,
) -> std::io::Result<()> {
    fn write(frame: CANFrame, channel: &str, logfile: &Path, note: &str) -> std::io::Result<()> {
        // TODO: Move formatting to seperate fn
        let mut formatted_data = "".to_owned();
        for item in frame.data() {
            formatted_data = format!("{}{:02X?} ", formatted_data, item);
        }
        let buffer: String = format!(
            "{0:<3} {1:<30} {2:<8} {3:<10} {4:<25}\n",
            note,
            Utc::now().naive_local().format("[%a %b %e %H:%M:%S %Y]:"),
            channel,
            format!("0x{:03X?}", frame.id()),
            formatted_data
        );

        let mut file = OpenOptions::new().append(true).create(true).open(logfile)?;
        file.write_all(buffer.as_bytes())?;
        Ok(())
    }

    match socket.read_frame() {
        Ok(frame) => {
            write(tx_frame, channel, logfile, "TX")?;
            write(frame, channel, logfile, "RX")
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn msg_processor_test() {
        let test_can_id: u32;
        let _test_can_msg: Vec<u8>;
        let test_msg_format = MsgFormat::new(
            String::from("TestMsgFormat#1"),
            Range {
                start: 0,
                end: 2_021,
            },
            vec![],
            2,
            vec![
                Section::new(
                    String::from("TestSec#1"),
                    1,
                    vec![
                        SubSec::new(String::from("TestSubSec#1"), 3, vec![1, 2], false, 0),
                        SubSec::new(String::from("TestSubSec#2"), 5, vec![5, 6], false, 0),
                    ],
                    false,
                    0,
                ),
                Section::new(
                    String::from("TestSec#2"),
                    1,
                    vec![
                        SubSec::new(String::from("TestSubSec#3"), 6, vec![1, 2], false, 0),
                        SubSec::new(String::from("TestSubSec#4"), 2, vec![], false, 0),
                    ],
                    false,
                    0,
                ),
            ],
            false,
            0,
        );
        test_msg_format.display();
        //println!("{}", test_msg_format);
        //println!("{:#?}", test_msg_format);
        for i in 0..test_msg_format.sections.len() {
            println!("<#-{}-#>", i + 1);
            test_msg_format.sections[i].display();
            test_msg_format.sections[i].display_sub_secs();
            println!("<#-{}-#>", i + 1);
        }
        println!("<#-END-#>");
        let width;
        let hex_cnt;
        test_can_id = random_cob_id_with_format(&test_msg_format);
        _test_can_msg = msg_processor(&test_msg_format);
        width = 12; //can_id typically expected to be <= 12 bits
        hex_cnt = (width) / 4;
        println!("--------");
        println!(
            "Returned msg_processor can_id (bin): {} bits\n{result:#0width$b}",
            width,
            result = test_can_id,
            width = width + 2
        );
        println!(
            "Returned msg_processor can_id (hex): {} hexits\n{result:#0width$X} ",
            hex_cnt,
            result = test_can_id,
            width = (hex_cnt as usize) + 2
        );
        println!("--------");
        //no longer prints out the test_can_msg, because that'd require re-converting
        //it from a Vector into a string of bits, will do if there's time
    }

    #[test]
    fn it_reads_and_writes_json() {
        // tempdir allows for reading and writing in a directory that will
        // be created and automatically deleted when the tempdir destructor
        // is run
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_output.json");
        let file_path_str = file_path.to_str().unwrap();

        //EMCY based test format
        let test_msg_format = MsgFormat::new(
            String::from("TestEMCYMsgFormat#1"),
            //0x080..0x0FF, EMCY COB-ID Range
            //https://en.wikipedia.org/wiki/CANopen#Predefined_Connection_Set[7]
            std::ops::Range {
                start: 0x080,
                end: 0x0FF,
            },
            vec![],
            3,
            vec![
                Section::new(
                    String::from("EmergencyErrorCode"),
                    2,
                    vec![
                        SubSec::new(String::from("EEC#1"), 8, vec![], false, 0),
                        SubSec::new(String::from("EEC#2"), 8, vec![], false, 0),
                    ],
                    false,
                    0,
                ),
                Section::new(
                    String::from("ErrorRegister"),
                    1,
                    vec![SubSec::new(String::from("ER#1"), 8, vec![], false, 0)],
                    false,
                    0,
                ),
                Section::new(
                    String::from("ManufacturerSpecificErrorCode"),
                    5,
                    vec![],
                    true,
                    0x00_00_00_00_00, //covering the space of 5 bytes
                ),
            ],
            false,
            0,
        );

        save_config(file_path_str, &test_msg_format).unwrap();
        let input_values = read_config(Path::new(file_path_str)).unwrap();
        assert_eq!(input_values, test_msg_format);
    }

    #[test]
    fn it_works_with_single_section_random_bytes() {
        let test_msg_format = MsgFormat::new(
            String::from("Test"),
            std::ops::Range {
                start: 0x080,
                end: 0x0FF,
            },
            vec![],
            1,
            vec![Section::new(String::from("Section"), 8, vec![], false, 0)],
            false,
            0,
        );

        // Ensure randomizer does not panic
        random_cob_id_with_format(&test_msg_format);
        msg_processor(&test_msg_format);
    }
}
