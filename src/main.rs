//use rand::rngs::ThreadRng; //required for passing in ThreadRng to funcs
use rand::Rng;

use socketcan::*;

//use std::str::FromStr;

use std::io::{self, Write};
use std::process::Command;

fn main() {
    //starting the random number generation thread here, rather
    //than invoking it for each run call of random_cob_id & random_msg
    //note to self: got this error before I reset rng = rand::thread_rng(),
    //because it seemed to need the value to be generated again
    //25 |     let mut rng = rand::thread_rng();
    //   |         ------- move occurs because `rng` has type `ThreadRng`, which does not implement the `Copy` trait
    //26 |
    //27 |     let test_cob_id = random_cob_id(rng);
    //   |                                     --- value moved here
    //28 |     let test_msg = random_msg(rng);
    //   |                               ^^^ value used here after move

    let test_cob_id = random_cob_id();
    let test_msg = random_msg();

    println!("{}", test_cob_id);
    println!("{:?}", test_msg);
    //note to self: got this error before I add ":?":
    //|     println!("{}", random_msg());
    //|                    ^^^^^^^^^^^^ `[u8; 8]` cannot be formatted with the default formatter
    //|
    //= help: the trait `std::fmt::Display` is not implemented for `[u8; 8]`
    //= note: in format strings you may be able to use `{:?}` (or {:#?} for pretty-print) instead
    //= note: required by `std::fmt::Display::fmt`

    let cs = CANSocket::open("vcan0").unwrap();
    create_frame_send_msg(cs, test_cob_id, &test_msg, false, false);

    create_bus("vcan1");
    destroy_bus("vcan1");

    //let mut user_in = Vec::new();
    //let socket_name: String;
    //let cob_id: u32;
    //let data: u8;
    //let rtr: bool;
    //let err: bool;

    ////disabling this section on input
    ////parsing the arguments into the vector user_in
    //for arg in std::env::args().skip(1) {
    //    //skipping name of program, arg(1)
    //    user_in.push(String::from_str(&arg).expect("error parsing argument"));
    //}

    ////if we want to specify the message length
    //if user_in.is_empty() || user_in.len() != 5 {
    //    error_exit();
    //} else if user_in.len() == 5 {
    //    socket_name = user_in[0].to_string();
    //    cob_id = u32::from_str_radix(&user_in[1], 16).expect("error parsing cob_id argument");
    //    data = u8::from_str_radix(&user_in[2], 16).expect("error parsing argument");
    //    rtr = bool::from_str(&user_in[3]).expect("error parsing rtr argument");
    //    err = bool::from_str(&user_in[4]).expect("error parsing err argument");

    //    let cs = CANSocket::open(&socket_name).unwrap();
    //    let test = [data];
    //    create_frame_send_msg(cs, cob_id, &test, rtr, err);
    //}
}

fn random_cob_id() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..2_021)
}

fn random_msg() -> [u8; 8] {
    let mut rng = rand::thread_rng();
    let mut data: [u8; 8] = [0; 8];
    //note to self: was getting this error before I added "= [0;8]"
    //|     data[0] = rng.gen_range(0..255);
    //|     ^^^^^^^ use of possibly-uninitialized `data`
    data[0] = rng.gen_range(0..255);
    data[1] = rng.gen_range(0..255);
    data[2] = rng.gen_range(0..255);
    data[3] = rng.gen_range(0..255);
    data[4] = rng.gen_range(0..255);
    data[5] = rng.gen_range(0..255);
    data[6] = rng.gen_range(0..255);
    data[7] = rng.gen_range(0..255);
    data
}

fn create_frame_send_msg(cs: CANSocket, cob_id: u32, data: &[u8], rtr: bool, err: bool) {
    //outputting a can message to the user chosen socket, with the given values
    let frame = CANFrame::new(cob_id, data, rtr, err).unwrap();
    cs.write_frame(&frame).unwrap();
}

/// Create a vcan bus using the following commands:
/// sudo ip link add dev <name> type vcan
/// sudo ip link set up <name>
/// This function will panic if errors are returned
fn create_bus(name: &str) {
    let output = Command::new("sudo")
        .arg("ip")
        .arg("link")
        .arg("add")
        .arg("dev")
        .arg(name)
        .arg("type")
        .arg("vcan")
        .output()
        .expect("failed to execute process");

    if !output.stderr.is_empty() {
        io::stderr().write_all(&output.stderr).unwrap();
        panic!("Unable to create bus {}, it may already be created", name)
    }

    let output = Command::new("sudo")
        .arg("ip")
        .arg("link")
        .arg("set")
        .arg(name)
        .arg("up")
        .output()
        .expect("failed to execute process");

    if !output.stderr.is_empty() {
        io::stderr().write_all(&output.stderr).unwrap();
        panic!("Unable to bring up bus {}", name)
    }
}

/// Destroy a vcan bus using the following commands:
/// sudo ip link del dev <name>
/// This function will panic if errors are returned
fn destroy_bus(name: &str) {
    let output = Command::new("sudo")
        .arg("ip")
        .arg("link")
        .arg("del")
        .arg("dev")
        .arg(name)
        .output()
        .expect("failed to execute process");

    if !output.stderr.is_empty() {
        io::stderr().write_all(&output.stderr).unwrap();
        panic!("Unable to destroy bus {}", name)
    }
}

//disabled just to avoid a warning for being unused
/*
fn error_exit() {
    eprintln!("Usage: cargo run SOCKET_NAME COB_ID CAN_MSG_DATA RTR ERR ... (String, u32::hex, u8::hex, bool, bool)");
    std::process::exit(1);
}
*/
