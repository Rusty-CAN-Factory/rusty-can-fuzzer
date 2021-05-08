use rand::Rng;

use socketcan::*;

use std::io::{self, Write};
use std::process::Command;

fn main() {
    let test_cob_id = random_cob_id();
    let test_msg = random_msg();

    println!("{}", test_cob_id);
    println!("{:?}", test_msg);

    let cs = CANSocket::open("vcan0").unwrap();
    create_frame_send_msg(cs, test_cob_id, &test_msg, false, false);

    create_bus("vcan1");
    destroy_bus("vcan1");
}

fn random_cob_id() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..2_021)
}

fn random_msg() -> [u8; 8] {
    let mut rng = rand::thread_rng();
    let mut data: [u8; 8] = [0; 8];
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
