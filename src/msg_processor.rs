//when I had this pub mod section, rust gave an error wanting me
//to import from "crate::mod_processor::mod_processor", so it
//seems if we don't explicitly declare a mod inside a file,
//that file's existence alone is what declares the module
//(aside from the "pub mod msg_processor" at the beginning of
//main.rs)
//pub mod msg_processor {
    use rand::Rng;
    use socketcan::*;
    use chrono::Utc;
    
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
//}
