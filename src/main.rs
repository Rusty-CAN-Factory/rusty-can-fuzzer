use std::process::Command;
use std::io::{self, Write};

fn main() {
    create_bus("vcan0");
    destroy_bus("vcan0");
}

fn create_bus(name: &str)
{
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

    if output.stderr.len() > 0
    {
        io::stderr().write_all(&output.stderr).unwrap();
        panic!("Unable create bus {}, it may already be created", name)
    }

    let output = Command::new("sudo")
    .arg("ip")
    .arg("link")
    .arg("set")
    .arg(name)
    .arg("up")
    .output()
    .expect("failed to execute process");

    if output.stderr.len() > 0
    {
        io::stderr().write_all(&output.stderr).unwrap();
        panic!("Unable to bring up bus {}", name)
    }
}

fn destroy_bus(name: &str)
{
    let output = Command::new("sudo")
    .arg("ip")
    .arg("link")
    .arg("del")
    .arg("dev")
    .arg(name)
    .output()
    .expect("failed to execute process");

    if output.stderr.len() > 0
    {
        io::stderr().write_all(&output.stderr).unwrap();
        panic!("Unable to destroy bus {}", name)
    }
}
