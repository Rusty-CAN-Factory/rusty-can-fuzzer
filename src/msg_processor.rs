use chrono::Utc;
use rand::Rng;
use socketcan::*;

pub struct SubSec<'a> {
    num_bits: u8,
    //holes: &[u8],
    holes: &'a [u8],
    is_specified: bool,
    specified_val: u8,
}

impl<'a> SubSec<'a> {
    pub fn display(self: &Self) {
        println!("num_bits {}, holes {:?}, \n\
                  is_specified {},  specified_val {}",
                  self.num_bits, self.holes, self.is_specified,
                  self.specified_val);
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
    section_proc();
    1
}

pub fn section_proc() -> u64 {
    let test_sub_sec = SubSec {
        num_bits: 3,
        holes: &[1,2],
        is_specified: false,
        specified_val: 0,
    };
    let result;
    println!("test_sub_sec values: ");
    test_sub_sec.display();
    println!();
    result = sub_sec_proc(test_sub_sec);
    println!("sub_sec_proc random result (dec): {} ", result);
    println!("sub_sec_proc random result (bin): {:#06b} ", result);
    1
}

pub fn sub_sec_proc(sub_sec: SubSec) -> u8 {
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
