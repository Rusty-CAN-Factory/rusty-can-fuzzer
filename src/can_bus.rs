use std::io::{self, Write};
use std::process::Command;

/// Create a vcan bus using the following commands:
/// sudo ip link add dev <name> type vcan
/// sudo ip link set up <name>
/// This function will panic if errors are returned
pub fn create_bus(name: &str) {
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
        //panic!("Unable to create bus {}, it may already be created", name)
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
pub fn destroy_bus(name: &str) {
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
