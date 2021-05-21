use chrono::Utc;
use rand::Rng;
use socketcan::*;

pub struct SubSec<'a> {
    name: &'a str,
    num_bits: u8,
    //holes: &[u8],
    holes: &'a [u8],
    is_specified: bool,
    specified_val: u8,
}

impl<'a> SubSec<'a> {
    pub fn display(self: &Self) {
        println!("{}: \n\
                  num_bits {}, holes {:?}, \n\
                  is_specified {},  specified_val {}",
                  self.name,
                  self.num_bits, self.holes, self.is_specified,
                  self.specified_val);
    }
}

pub struct Section<'a> {
    name: &'a str,
    num_bytes: u8,
    sub_secs: &'a [SubSec<'a>],
    is_specified: bool,
    specified_val: u8,
}

impl<'a> Section<'a> {
    pub fn display(self: &Self) {
        println!("{}: \n\
                  num_bytes {}, is_specified {}, \n\
                  specified_val {}",
                  self.name,
                  self.num_bytes, self.is_specified,
                  self.specified_val);
    }

    pub fn display_sub_secs(self: &Self) {
        //let mut i = 0;
        //for i in self.sub_secs.len() {
        for i in 0..self.sub_secs.len() {
            println!("SubSec #{}: ", i);
            self.sub_secs[i].display();
        }
    }
}

pub fn random_cob_id() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..2_021)
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

pub fn msg_processor() -> u64 {
    let sec_result;
    let test_section = Section {
        name: "TestSec#1",
        num_bytes: 1,
        sub_secs: &[
            SubSec {
                name: "TestSubSec#1",
                num_bits: 3,
                holes: &[1,2],
                is_specified: false,
                specified_val: 0,
            },
            SubSec {
                name: "TestSubSec#2",
                num_bits: 5,
                holes: &[5,6],
                is_specified: false,
                specified_val: 0,
            },
        ],
        is_specified: false,
        specified_val: 0,
    };
    sec_result = section_proc(&test_section);
    //note to self:
    //needs to be adjusted to change the X in ":#Xb" to fit the
    //number of bits/bytes in a section, it seems having 2 more
    //the number of bits helps (leaves room for "0b")
    println!("Complete section result (bin): {:#10b} ", sec_result);
    1
}

pub fn section_proc(section: &Section) -> u64 {
    //let test_sub_sec = SubSec {
    //    name: "TestSubSec#0",
    //    num_bits: 3,
    //    holes: &[1,2],
    //    is_specified: false,
    //    specified_val: 0,
    //};
    let mut prev_sub_sec_result = 0;
    let mut sub_sec_result;
    let mut result = 0;
    let mut has_looped = false;
    for i in 0..section.sub_secs.len() {
        println!("########");
        println!("test_sub_sec values: ");
        section.sub_secs[i].display();
        println!();
        sub_sec_result = sub_sec_proc(&section.sub_secs[i]);
        println!("sub_sec_proc result (dec): {} ", sub_sec_result);
        println!("sub_sec_proc result (bin): {:#10b} ", sub_sec_result);
        //MAY NOT NEED THIS CHECK, since the above for loop should take care of it
        //if(section.sub_secs[i+1].exists()) {
        if i < section.sub_secs.len() && has_looped {
            //shifting the bits to make room for the new result
            result = prev_sub_sec_result << section.sub_secs[i].num_bits;
        }
        //ORing to add the new result on the end
        result = result | sub_sec_result;
        println!("Current section_proc result (bin): {:#10b} ", result);
        prev_sub_sec_result = sub_sec_result;
        has_looped = true;
    }
    result as u64
}

pub fn sub_sec_proc(sub_sec: &SubSec) -> u8 {
    let mut rng = rand::thread_rng();
    //-2 instead of -1 because for example 3^2-1=8,
    //but we can only fit up to 7 in 3 bits
    let range = sub_sec.num_bits.pow(2)-2;
    let mut loop_continue = true;
    let mut result = 0;
    println!("sub_sec_proc range: {} ", range);
    while loop_continue == true {
        result = rng.gen_range(0..range);
        if sub_sec.holes.contains(&result) {
            println!("Fell in a hole!\t\
                      Random result {}, Holes {:?}",
                      result, sub_sec.holes);
        }
        else {
            loop_continue = false
        }
    }
    result
}
