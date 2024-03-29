pub mod can_bus;
pub mod msg_processor;
use can_bus::*;
use msg_processor::*;
use socketcan::*;
#[macro_use]
extern crate clap;
use clap::{App, Arg};
use rand::seq::SliceRandom;
use std::path::Path;
use std::process;
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
                .help("Use a randomly generated ID")
                .conflicts_with_all(&["id", "message_format"]),
        )
        .arg(
            Arg::with_name("random_message")
                .long("random-message")
                .help("Use a randomly generated message")
                .conflicts_with_all(&["message", "message_format"]),
        )
        .arg(
            Arg::with_name("message_format")
                .short("f")
                .long("message-format")
                .value_name("FILE|DIR")
                .takes_value(true)
                .help("Use a provided message format json file")
                .conflicts_with_all(&["random_message", "random_id", "message"]),
        )
        .arg(
            Arg::with_name("listen_mode")
                .short("l")
                .long("listen")
                .help(
                    "Enable listen mode to log responses during delay, delay is shortened \
                       when responses are heard",
                ),
        )
        .arg(
            Arg::with_name("listen_log")
                .long("listen-log")
                .takes_value(true)
                .default_value("log.txt")
                .value_name("FILE")
                .help("Listen mode log file for storing responses"),
        )
        .get_matches();

    let channels: Vec<String> = matches
        .values_of("channels")
        .unwrap()
        .map(String::from)
        .collect();
    let delay: u64 = matches
        .value_of("delay")
        .unwrap()
        .parse()
        .expect("Unable to parse delay value, should be a positive integer value");
    let mut id: u32 = u32::from_str_radix(matches.value_of("id").unwrap(), 16)
        .expect("Unable to parse id, should be a 32bit integer value");
    let destroy: bool = !matches.is_present("destroy");
    let mut message_parsed: Vec<u8> = matches
        .values_of("message")
        .unwrap()
        .map(|x| {
            u8::from_str_radix(x, 16)
                .expect("Unable to parse message value, should be 1 byte hex value")
        })
        .collect();

    let mut repeat: i64 = match matches.value_of("repeat").unwrap_or("1").parse() {
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

    let msg_formats: Option<Vec<MsgFormat>> = matches
        .value_of("message_format")
        .map(|s| read_configs(Path::new(&s)).unwrap());

    // Create Handler for keyboard interrupt signal
    // This will cleanup bus when exiting with control+c
    let channel_clone = channels.clone();
    ctrlc::set_handler(move || {
        if destroy {
            for channel in &channel_clone {
                destroy_bus(channel)
            }
        }
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let listen_mode = matches.is_present("listen_mode");
    let listen_log = Path::new(matches.value_of("listen_log").unwrap());

    // Setup bus and socket objects
    let mut sockets = Vec::new();
    for channel in &channels {
        create_bus(channel);
        sockets.push((CANSocket::open(channel).unwrap(), channel));
    }

    let delay_seconds = time::Duration::from_secs(delay);

    // In listen mode read timeout is used as delay
    if listen_mode {
        for socket in &sockets {
            socket.0.set_read_timeout(delay_seconds).unwrap();
        }
    }
    // Print Banner Message
    println!(
        "{0:<30} {1:<8} {2:<10} {3:<25}",
        "Timestamp", "Channel", "COB ID", "Message"
    );
    println!("{:-<75}", "");

    // Main app loop
    while repeat != 0 {
        for socket in &sockets {
            if msg_formats.is_some() {
                let formats = msg_formats.as_ref().unwrap();
                // Choose random format when multiple provided
                let format = &formats.choose(&mut rand::thread_rng()).unwrap();

                id = random_cob_id_with_format(&format);
                message_parsed = msg_processor(&format);
            } else {
                if random_id {
                    id = random_cob_id()
                }
                if random_message {
                    message_parsed = random_msg();
                }
            }

            let frame =
                create_frame_send_msg(&socket.0, &socket.1, id, &message_parsed, false, false);
            if listen_mode {
                if let Ok(f) = frame {
                    listen(&socket.0, &socket.1, listen_log, f).unwrap()
                }
            }
        }

        if repeat != -1 {
            repeat -= 1;
        }

        // listen mode uses blocking read to add delay
        if !listen_mode {
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
