use chrono::Utc;
use rand::Rng;
use socketcan::*;
use core::ops::Range; //for COB_ID range
//https://stackoverflow.com/a/44690529 //for printing as binary
//https://stackoverflow.com/a/55494303 //for adding underscores to bin out

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
                  is_specified {}, specified_val {}",
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
    specified_val: u64,
}

impl<'a> Section<'a> {
    pub fn display(self: &Self) {
        println!("{}: \n\
                  num_bytes {}, is_specified {}, \
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

pub struct MsgFormat<'a> {
    name: &'a str,
    cob_id_range: Range<u64>,
    num_sections: u8,
    sections: &'a [Section<'a>],
    is_specified: bool,
    specified_val: u8,
}

impl<'a> MsgFormat<'a> {
    pub fn display(self: &Self) {
        println!("{}: \n\
                  cob_id_range: {:?}, num_sections {}, \n\
                  is_specified {}, specified_val {}",
                  self.name,
                  self.cob_id_range, self.num_sections,
                  self.is_specified, self.specified_val);
    }

    pub fn display_sections(self: &Self) {
        for i in 0..self.sections.len() {
            println!("Section #{}: ", i);
            self.sections[i].display();
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

//returns a tuple, COB_ID and MSG
//tentative
//pub fn msg_processor(msg_format: &MsgFormat) -> (u64,u64) {
pub fn msg_processor() -> (u64,u64) {
    let test_msg_format = MsgFormat {
        name: "TestMsgFormat#1",
        cob_id_range: { 0..9999 },
        num_sections: 2,
        sections: &[
            Section {
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
            },
            Section {
                name: "TestSec#2",
                num_bytes: 1,
                sub_secs: &[
                    SubSec {
                        name: "TestSubSec#3",
                        num_bits: 6,
                        holes: &[1,2],
                        is_specified: false,
                        specified_val: 0,
                    },
                    SubSec {
                        name: "TestSubSec#4",
                        num_bits: 2,
                        holes: &[],
                        is_specified: false,
                        specified_val: 0,
                    },
                ],
                is_specified: false,
                specified_val: 0,
            },
        ],
        is_specified: false,
        specified_val: 0,
    };
    //let test_section = Section {
    //    name: "TestSec#1",
    //    num_bytes: 1,
    //    sub_secs: &[
    //        SubSec {
    //            name: "TestSubSec#1",
    //            num_bits: 3,
    //            holes: &[1,2],
    //            is_specified: false,
    //            specified_val: 0,
    //        },
    //        SubSec {
    //            name: "TestSubSec#2",
    //            num_bits: 5,
    //            holes: &[5,6],
    //            is_specified: false,
    //            specified_val: 0,
    //        },
    //    ],
    //    is_specified: false,
    //    specified_val: 0,
    //};
    let mut prev_sec_result = 0;
    let mut sec_result;
    let mut result = 0;
    let mut has_looped = false;
    let mut width;
    for i in 0..test_msg_format.sections.len() {
        println!("<#-{}-#>", i+1);
        //println!("test_section values: ");
        test_msg_format.sections[i].display();
        println!();
        sec_result = section_proc(&test_msg_format.sections[i]);
        //println!("section_proc result (dec): {} ", sec_result);
        //println!("section_proc result (bin): {:#10b} ", sec_result);
        //MAY NOT NEED THIS CHECK, since the above for loop should take care of it
        //if(section.sub_secs[i+1].exists()) {
        if i < test_msg_format.sections.len() && has_looped {
            //shifting the bits to make room for the new result
            result = prev_sec_result << test_msg_format.sections[i].num_bytes*8;
        }
        //ORing to add the new result on the end
        result = result | sec_result;
        println!("<#-{}-#>", i+1);
        width = (i+1)*8;
        //println!("\tCurrent msg_processor result (bin): {:#10b} ", result);
        println!("\tCurrent msg_processor result (bin): {} bits\n\t{result:#0width$b} ",
                 width, result=result, width=width+2);
        prev_sec_result = sec_result;
        has_looped = true;
    }
    println!("<#-END-#>");
    let width = test_msg_format.sections.len()*8;
    //note to self:
    //it seems having 2 more the number of bits helps (leaves room for "0b")
    //(needs more testing to be sure)
    println!("Complete msg_processor result (bin): {} bits\n{result:#0width$b}",
             width, result=result, width=width+2);
    (random_cob_id() as u64, result as u64)
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
    let mut bit_cnt = 0;
    let mut hex_cnt;
    for i in 0..section.sub_secs.len() {
        bit_cnt += section.sub_secs[i].num_bits;
        println!("##{}", i+1);
        //println!("test_sub_sec values: ");
        section.sub_secs[i].display();
        println!();
        sub_sec_result = sub_sec_proc(&section.sub_secs[i]);
        //MAY NOT NEED THIS CHECK, since the above for loop should take care of it
        //if(section.sub_secs[i+1].exists()) {
        if i < section.sub_secs.len() && has_looped {
            //shifting the bits to make room for the new result
            result = prev_sub_sec_result << section.sub_secs[i].num_bits;
        }
        //ORing to add the new result on the end
        result = result | sub_sec_result;
        println!("##{}", i+1);
        //println!("\tCurrent section_proc result (bin): {:#10b} ", result);
        //println!("\tCurrent section_proc result (bin): {} bits |{result:#width$b} ",
        //         (i+1)*4, result=result, width=(i+1)*8+2);
        //println!("\tCurrent section_proc result (bin): {} bits |{result:#width$b} ",
        //         section.sub_secs[i].num_bits, result=result, 
        //         width=(section.sub_secs[i].num_bits as usize)+2);
        println!("\tCurrent section_proc result (bin): {} bits |{result:#0width$b} ",
                 bit_cnt, result=result, width=(bit_cnt as usize)+2);
        //need "+2" because if for example bit_cnt is 1, dividing it by 4 results in 0
        //(a 1 bit value is surely still a 1 hexit value
        hex_cnt = (bit_cnt+2)/4;
        //println!("\tCurrent section_proc result (hex): {:#10X} ", result);
        //println!("\tCurrent section_proc result (hex): {} hexits |{result:#0width$X} ",
        //         (bit_cnt/4), result=result, width=(bit_cnt/4) as usize);
        println!("\tCurrent section_proc result (hex): {} hexits |{result:#0width$X} ",
                 hex_cnt, result=result, width=(hex_cnt as usize)+2);
        prev_sub_sec_result = sub_sec_result;
        has_looped = true;
    }
    result as u64
}

pub fn sub_sec_proc(sub_sec: &SubSec) -> u8 {
    let mut rng = rand::thread_rng();
    //-2 instead of -1 because for example 3^2-1=8,
    //but we can only fit up to 7 in 3 bits
    //let range = sub_sec.num_bits.pow(2)-2;
    //this seems to have been because I screwed up and
    //mixed up the base and the exponent
    //tentative fix
    let range = 2_u8.pow(sub_sec.num_bits as u32)-1;
    //first generation
    let mut result = rng.gen_range(0..range);
    println!("sub_sec_proc range: {} ", range);
    while sub_sec.holes.contains(&result) {
        println!("Fell in a hole!\t\
                  Random result {}, Holes {:?}",
                  result, sub_sec.holes);
        result = rng.gen_range(0..range);
    }
    println!("\tsub_sec_proc result (dec): {} ", result);
    //println!("sub_sec_proc result (bin): {:#10b} ", sub_sec_result);
    println!("\tsub_sec_proc result (bin): {} bits |{result:#0width$b} ",
             sub_sec.num_bits, result=result, width=(sub_sec.num_bits as usize)+2);
    result
}
