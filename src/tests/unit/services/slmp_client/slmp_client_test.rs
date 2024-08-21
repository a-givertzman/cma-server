#[cfg(test)]

mod slmp_client {
    use log::{warn, debug};
    use std::{io::{self, Read, Write}, net::TcpStream, sync::Once, thread, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::services::slmp_client::slmp::{c_slmp_const::FrameType, device_code::DeviceCode, slmp_packet::SlmpPacket};
    ///
    ///
    static INIT: Once = Once::new();
    ///
    /// once called initialisation
    fn init_once() {
        INIT.call_once(|| {
            // implement your initialisation code to be called only once for current test file
        })
    }
    ///
    /// returns:
    ///  - ...
    fn init_each() -> () {}
    ///
    /// Testing read from SLMP device code D
    #[ignore = "Manual test with phisical device"]
    #[test]
    fn read_d() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(1000));
        test_duration.run().unwrap();
        let slmp_packet = SlmpPacket::new(self_id, DeviceCode::D, 1112, 4);
        match slmp_packet.read_packet(FrameType::BinReqSt) {
            Ok(read_request) => {
                debug!("read request: {:02X?}", read_request);
                loop {
                    match TcpStream::connect("192.168.120.200:4999") {
                        Ok(mut stream) => {
                            loop {
                                match stream.write_all(&read_request) {
                                    Ok(_) => {
                                        print!("\t- Ok");
                                        let mut reply = [0; 32];
                                        match stream.read(&mut reply) {
                                            Ok(_) => {
                                                println!("\treply: {:?}", reply);
                                                // const I16_SIZE: usize = 2;
                                                // const I32_SIZE: usize = 4;
                                                const FLOAT_SIZE: usize = 4;
                                                const FLOAT_COUNT: usize = 3;
                                                const DATA_OFFSET: usize = 11; // for header skip
                                                const DATA_SIZE: usize = FLOAT_COUNT * FLOAT_SIZE;
                                                let data_without_header = &reply[DATA_OFFSET..(DATA_OFFSET + DATA_SIZE)];
                                                println!("\tdata_without_header: {:?}", data_without_header);
                                                let floats = data_without_header
                                                    .chunks_exact(FLOAT_SIZE)
                                                    .map(|chunk| {
                                                        let bytes: [u8; FLOAT_SIZE] = chunk.try_into().unwrap();
                                                        f32::from_le_bytes(bytes)
                                                    })
                                                    .collect::<Vec<f32>>();
                                                println!("\tfloats: {:?}", floats);
                                                // let data_without_header = &reply[(DATA_OFFSET + DATA_SIZE)..(DATA_OFFSET + DATA_SIZE + I16_SIZE)];
                                                // println!("\tdata_without_header: {:?}", data_without_header);
                                                // let ints = data_without_header
                                                //     .chunks_exact(I16_SIZE)
                                                //     .map(|chunk| {
                                                //         let bytes: [u8; I16_SIZE] = chunk.try_into().unwrap();
                                                //         i16::from_le_bytes(bytes)
                                                //     })
                                                //     .collect::<Vec<i16>>();
                                                // println!("\tints: {:?}", ints);
                                                // let data_without_header = &reply[(DATA_OFFSET + DATA_SIZE+ I16_SIZE)..(DATA_OFFSET + DATA_SIZE + I16_SIZE + I32_SIZE)];
                                                // println!("\tdata_without_header: {:?}", data_without_header);
                                                // let ints = data_without_header
                                                //     .chunks_exact(I32_SIZE)
                                                //     .map(|chunk| {
                                                //         let bytes: [u8; I32_SIZE] = chunk.try_into().unwrap();
                                                //         i32::from_le_bytes(bytes)
                                                //     })
                                                //     .collect::<Vec<i32>>();
                                                // println!("\tbig ints: {:?}", ints);
                                            },
                                            Err(err) => warn!("Tcp read error: {:#?}", err),
                                        }
                                    }
                                    Err(err) => warn!("Tcp send error: {:#?}", err),
                                }
                                thread::sleep(Duration::from_millis(1000));
                            }
                        },
                        Err(err) => warn!("Tcp connection error: {:#?}", err),
                    }
                    thread::sleep(Duration::from_millis(1000));
                }
            }
            Err(err) => warn!("Build write request error:: {:#?}", err),
        }
        // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        test_duration.exit();
    }
    // fn to_real() {
    //     const FLOAT_SIZE: usize = 4;
    //     const FLOAT_COUNT: usize = 3;
    //     const DATA_OFFSET: usize = 11; // for header skip
    //     const DATA_SIZE: usize = FLOAT_COUNT * FLOAT_SIZE;
    //     let data_without_header = &reply[DATA_OFFSET..(DATA_OFFSET + DATA_SIZE)];
    //     println!("\tdata_without_header: {:?}", data_without_header);
    //     let floats = data_without_header
    //         .chunks_exact(FLOAT_SIZE)
    //         .map(|chunk| {
    //             let bytes: [u8; 4] = chunk.try_into().unwrap();
    //             f32::from_le_bytes(bytes)
    //         })
    //         .collect::<Vec<f32>>();
    //     println!("\tfloats: {:?}", floats);
    // }
    ///
    /// Testing write to SLMP device code D
    #[ignore = "Manual test with phisical device"]
    #[test]
    fn write_d() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(1000));
        test_duration.run().unwrap();
        let slmp_packet = SlmpPacket::new(self_id, DeviceCode::D, 1106, 2);
        loop {
            match TcpStream::connect("192.168.120.200:4999") {
                Ok(mut stream) => {
                    loop {
                        let write_bytes = &(-32768i16).to_le_bytes();
                        debug!("write bytes: {:02X?}", write_bytes);
                        match slmp_packet.write_packet(FrameType::BinReqSt, write_bytes) {
                            Ok(write_request) => {
                                debug!("write request: {:02X?}", write_request);
                                match stream.write_all(&write_request) {
                                    Ok(_) => {
                                        println!("\t write - Ok");
                                        io::stdout().flush().unwrap();
                                    }
                                    Err(err) => warn!("Tcp send error: {:#?}", err),
                                }
                                thread::sleep(Duration::from_millis(1000));
                            },
                            Err(err) => warn!("Build write request error:: {:#?}", err),
                        }
                    }
                },
                Err(err) => warn!("Tcp connection error: {:#?}", err),
            }
            thread::sleep(Duration::from_millis(1000));
        }
        // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        // test_duration.exit();
    }
}
