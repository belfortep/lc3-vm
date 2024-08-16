use std::{
    fs,
    io::{stdin, BufRead},
    os::unix::net::UnixDatagram,
};

use lc3_vm::constants::{CLIENT_PATH, SERVER_PATH, STREAM_DATA_SEPARATOR};

fn print_instructions() {
    println!("Instructions: ");
    println!("<n> to execute one instruction");
    println!("<a number> to execute that number of instructions (recommended)");
    println!("<r> to print the state of the registers of the program");
    println!("remember to use the program you are debugging if it needs user input");
}

fn main() -> Result<(), String> {
    let _ = fs::remove_file(CLIENT_PATH);
    let socket = UnixDatagram::bind(CLIENT_PATH).map_err(|error| error.to_string())?;
    print_instructions();
    for line in stdin().lock().lines() {
        let line = line.map_err(|error| error.to_string())?;
        socket
            .send_to(line.as_bytes(), SERVER_PATH)
            .map_err(|error| error.to_string())?;
        let mut buffer = [0; 1024];
        match socket.recv(&mut buffer) {
            Ok(size) => {
                let response = String::from_utf8_lossy(&buffer[..size]);
                let response = response.split(STREAM_DATA_SEPARATOR);
                for data in response {
                    println!("{}", data);
                }
            }
            Err(_) => {
                println!("Couldn't receive");
            }
        }
    }

    Ok(())
}
