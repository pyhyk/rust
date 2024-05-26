use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::stdin;
use std::io::Write;
use std::net::UdpSocket;
use std::process::exit;
use std::time::{Duration, Instant};

// #[derive(Hash)]
fn calculate_checksum<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

// #[derive(Hash)]
fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:12346")?;
    println!("Listening on {}", socket.local_addr().unwrap());

    socket.connect("127.0.0.1:12345")?;
    println!("Connection has been established.");
    socket.set_read_timeout(Some(Duration::from_secs(5)))?;

    // let windows_max = 5;
    // let mut windows = Vec::new()
    let mut sequence_number:u32 = 0;
    println!("please enter the message you want to send to the server :");
    loop {
        // let data = b"Hello, server!";
        std::io::stdout().flush()?;

        println!("enter 'exit' to quit the program");
        let mut data_untrim = String::new();
        stdin().read_line(&mut data_untrim)?;
        let data = data_untrim.trim();

        // if data.trim() == "exit" {
        //     println!("bye bye!");
        //     exit(1);
        // }
        //calculate checksum
        let checksum = calculate_checksum(&data);
        println!("the checksum is {:?}", checksum);

        //modify the data
        // let data = data.trim();
        // let data = data.as_bytes();

        //append checksum to data
        let mut data_with_checksum = data.as_bytes().to_vec();
        data_with_checksum.extend(&checksum.to_be_bytes());
        // socket.send(&data_with_checksum)?;

        data_with_checksum.extend(&sequence_number.to_be_bytes());

        let mut buf = [0; 1024];
        let mut attempts = 0;
        let max_attempts = 5;
        let mut ack_received = false;

        while attempts < max_attempts && !ack_received {
            let start = Instant::now();

            // Send data
            socket.send(&data_with_checksum)?;

            // Wait for ACK
            while start.elapsed() < Duration::from_secs(1) {
                match socket.recv(&mut buf) {
                    Ok(amt) => {
                        let ack = &buf[..amt];
                        if ack == b"ACK" {
                            println!("Received ACK");
                            ack_received = true;
                            if data.trim() == "exit" {
                                println!("bye bye!");
                                exit(1);
                            }
                            break;
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No data received, continue waiting
                    }
                    Err(e) => {
                        eprintln!("Error receiving ACK: {:?}", e);
                        break;
                    }
                }
            }

            if !ack_received {
                attempts += 1;
                println!("No ACK received, retrying... (attempt {})", attempts);
            }
        }

        if !ack_received {
            println!("Failed to receive ACK after {} attempts", max_attempts);
        }

        sequence_number += 1;
        
    }
}
