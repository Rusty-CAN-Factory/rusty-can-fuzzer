use chrono::Utc;
use core::ops::Range;
use rand::Rng;
use socketcan::*; //for COB_ID range

#[derive(Debug)]
pub struct SubSec {
    name: String,
    num_bits: u8,
    holes: Vec<u8>,
    is_specified: bool,
    specified_val: u8,
}

impl SubSec {
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

    pub fn display(&self) {
        println!(
            "\t\t{}: \n\
                  \t\tnum_bits {}, holes {:?}, \n\
                  \t\tis_specified {}, specified_val {}",
            self.name, self.num_bits, self.holes, self.is_specified, self.specified_val
        );
    }
}

#[derive(Debug)]
pub struct Section {
    name: String,
    num_bytes: u8,
    sub_secs: Vec<SubSec>,
    is_specified: bool,
    specified_val: u64,
}

impl Section {
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

    pub fn display(&self) {
        println!(
            "\t{}: \n\
                  \tnum_bytes {}, is_specified {}, specified_val {}",
            self.name, self.num_bytes, self.is_specified, self.specified_val
        );
    }

    pub fn display_sub_secs(&self) {
        for i in 0..self.sub_secs.len() {
            println!("\t\tSubSec #{}: ", i);
            self.sub_secs[i].display();
        }
    }
}

#[derive(Debug)]
pub struct MsgFormat {
    name: String,
    cob_id_range: Range<u32>,
    num_sections: u8,
    sections: Vec<Section>,
    is_specified: bool,
    specified_val: u64,
}

impl MsgFormat {
    pub fn new(
        name: String,
        cob_id_range: Range<u32>,
        num_sections: u8,
        sections: Vec<Section>,
        is_specified: bool,
        specified_val: u64,
    ) -> Self {
        Self {
            name,
            cob_id_range,
            num_sections,
            sections,
            is_specified,
            specified_val,
        }
    }
    pub fn display(&self) {
        println!(
            "{}: \n\
                  cob_id_range: {:?}, num_sections {}, \n\
                  is_specified {}, specified_val {}",
            self.name, self.cob_id_range, self.num_sections, self.is_specified, self.specified_val
        );
    }

    pub fn display_sections(&self) {
        for i in 0..self.sections.len() {
            println!("\tSection #{}: ", i);
            self.sections[i].display();
        }
    }
}

pub fn random_cob_id(msg_format: &MsgFormat) -> u32 {
    let mut rng = rand::thread_rng();
    //total range for cob_id in CANOpen is 0..2_021 (aka 0x0..0x7E5)
    //https://en.wikipedia.org/wiki/CANopen#Predefined_Connection_Set[7]
    rng.gen_range(msg_format.cob_id_range.start..msg_format.cob_id_range.end)
}

pub fn random_msg() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let data: Vec<u8> = (0..8).map(|_| rng.gen_range(0..255)).collect();
    data
}

//outputting a can message to the user chosen socket, with the given values
pub fn create_frame_send_msg(
    cs: &CANSocket,
    channel: &str,
    cob_id: u32,
    data: &[u8],
    rtr: bool,
    err: bool,
) {
    let frame = CANFrame::new(cob_id, data, rtr, err).unwrap();
    cs.write_frame(&frame).unwrap();
    println!(
        "{0:<30} {1:<8} {2:<10} {3:<25}",
        Utc::now().naive_local().format("[%a %b %e %H:%M:%S %Y]:"),
        channel,
        format!("0x{:03X?}", cob_id),
        format!("{:02X?}", data)
    );
}

//returns a Vector of u8 chunks containing the message
pub fn msg_processor(msg_format: &MsgFormat) -> Vec<u8> {
    let mut sec_result;
    let mut result = 0;
    let msg_byte_array;
    let mut msg_byte_vec: Vec<u8> = Vec::new();
    let mut total_num_bytes = 0;
    for i in 0..msg_format.sections.len() {
        sec_result = section_proc(&msg_format.sections[i]);
        //shifting the bits to make room for the new result
        result <<= msg_format.sections[i].num_bytes * 8;
        //ORing to add the new result on the end
        result |= sec_result;
        total_num_bytes += msg_format.sections[i].num_bytes;
    }
    //bit shifting the final result so we push the actual
    //code all the way to the left as needed for CAN
    result <<= 64 - total_num_bytes * 8;
    //Chopping up result into a vec<u8>!
    //(done at end because it's simpler to do bit shifting with a single number before now)
    msg_byte_array = result.to_be_bytes();
    //clippy wanted us to change this, to use the more "Rusty" for loop
    //style than my more C/C++ style using i
    for msg in &msg_byte_array {
        msg_byte_vec.push(*msg); //de-referencing like with array[i] before
    }
    msg_byte_vec
}

pub fn section_proc(section: &Section) -> u64 {
    let mut sub_sec_result;
    let mut result: u64 = 0;
    if section.is_specified {
        return section.specified_val;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn msg_processor_test() {
        let test_can_id: u32;
        let test_can_msg: Vec<u8>;
        let test_msg_format = MsgFormat::new(
            String::from("TestMsgFormat#1"),
            Range {
                start: 0,
                end: 2_021,
            },
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
        let mut width;
        let mut hex_cnt;
        test_can_id = random_cob_id(&test_msg_format);
        test_can_msg = msg_processor(&test_msg_format);
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
}
