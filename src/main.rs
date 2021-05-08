use rand::Rng;
use socketcan::*;
use std::io::{self, Write};
use std::process::Command;
#[macro_use]
extern crate clap;
use chrono::Utc;
use clap::{App, Arg};
use std::{thread, time};

fn main() {
    let matches = App::new("Rusty Can Fuzzer")
        .version("0.1")
        .author(crate_authors!())
        .about("A CAN Bus fuzzer CLI written in rust")
        .arg(
            Arg::with_name("channels")
                .short("c")
                .long("channels")
                .value_name("CHANNEL")
                .help("The channel to create and send CAN messages on")
                .takes_value(true)
                .multiple(true)
                .default_value("vcan0"),
        )
        .arg(
            Arg::with_name("delay")
                .short("d")
                .long("delay")
                .value_name("DELAY")
                .help("Adjust the message-send delay time, used in conjunction with -r")
                .takes_value(true)
                .default_value("1"),
        )
        .arg(
            Arg::with_name("id")
                .short("i")
                .long("id")
                .value_name("ID")
                .help("The COB ID to use for the messages")
                .takes_value(true)
                .default_value("10"),
        )
        .arg(
            Arg::with_name("destroy")
                .short("n")
                .long("no-destroy")
                .help("Stop rusty-can-dev from destroying the channel at end of life")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("message")
                .short("m")
                .long("message")
                .value_name("BYTE")
                .help("The 8 bytes to send as the CAN message")
                .multiple(true)
                .takes_value(true)
                .default_value("1"),
        )
        .arg(
            Arg::with_name("repeat")
                .short("r")
                .long("repeat")
                .value_name("N")
                .help(
                    "Repeat sending the message N times or -1 for infinite times, \
                     every so often defined by -d, using in conjunction with -d",
                )
                .takes_value(true)
                .allow_hyphen_values(true),
        )
        .arg(
            Arg::with_name("random_id")
                .long("random-id")
                .help("Use a randomly generated ID (this disables -i)"),
        )
        .arg(
            Arg::with_name("random_message")
                .long("random-message")
                .help("Use a randomly generated message (this disables -m)"),
        )
        .get_matches();

    let channels: Vec<&str> = matches.values_of("channels").unwrap().collect();
    let delay: u64 = matches
        .value_of("delay")
        .unwrap()
        .parse()
        .expect("Unable to parse delay value, should be a positive integer value");
    let id: u32 = u32::from_str_radix(matches.value_of("id").unwrap(), 16)
        .expect("Unable to parse id, should be a 32bit integer value");
    let destroy: bool = !matches.is_present("destroy");
    let message_parsed: Vec<u8> = matches
        .values_of("message")
        .unwrap()
        .map(|x| {
            u8::from_str_radix(x, 16)
                .expect("Unable to parse message value, should be 1 byte hex value")
        })
        .collect();

    let repeat: i64 = match matches.value_of("repeat").unwrap_or("1").parse() {
        Ok(v) if v < -1 => panic!(
            "Unable to parse repeat value, should be a postitive integer value \
             (or -1 for infinite repeat), {} provided",
            v
        ),
        Ok(v) => v,
        Err(e) => panic!("Unable to parse repeat value: {}", e),
    };

    let random_id: bool = matches.is_present("random_id");
    let random_message: bool = matches.is_present("random_message");

    if random_id || random_message {
        panic!("Random id and random message not yet implemented!")
    }

    //Setup bus and socket objects
    let mut sockets = Vec::new();
    for channel in &channels {
        create_bus(channel);
        sockets.push((CANSocket::open(channel).unwrap(), channel));
    }

    let delay_seconds = time::Duration::from_secs(delay);

    // Print Banner Message
    println!(
        "{0:<30} {1:<8} {2:<10} {3:<25}",
        "Timestamp", "Channel", "COB ID", "Message"
    );
    println!("{:-<73}", "");

    // Send messages repeat times
    for _ in 0..repeat {
        for socket in &sockets {
            create_frame_send_msg(&socket.0, &socket.1, id, &message_parsed, false, false);
        }
        thread::sleep(delay_seconds);
    }

    // Send messages infinite times when repeat is -1
    if repeat == -1 {
        loop {
            for socket in &sockets {
                create_frame_send_msg(&socket.0, &socket.1, id, &message_parsed, false, false);
            }
            thread::sleep(delay_seconds);
        }
    }

    // Tear down bus
    if destroy {
        for channel in &channels {
            destroy_bus(channel)
        }
    }
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

//outputting a can message to the user chosen socket, with the given values
fn create_frame_send_msg(
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
