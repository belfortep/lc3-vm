use std::{
    io::{stdin, BufRead, BufReader, Write},
    net::TcpStream,
};
const LOCAL_HOST: &str = "127.0.0.1";
const PORT: &str = "3000";

fn print_instructions() {
    println!("Instructions: ");
    println!("<n> to execute one instruction");
    println!("<a number> to execute that number of instructions (recommended)");
    println!("<r> to print the state of the registers of the program");
    println!("remember to use the program you are debugging if it needs user input");
}

fn main() -> Result<(), String> {
    let connection_address = format!("{}:{}", LOCAL_HOST, PORT);
    let stream = TcpStream::connect(connection_address).map_err(|error| error.to_string())?;
    print_instructions();
    let mut connection_reader = BufReader::new(&stream);
    for line in stdin().lock().lines() {
        let line = line.map_err(|error| error.to_string())?;
        writeln!(&stream, "{}", line).map_err(|error| error.to_string())?;
        let mut response = String::new();
        connection_reader
            .read_line(&mut response)
            .map_err(|error| error.to_string())?;
        let response = response.split("::");
        for data in response {
            println!("{}", data);
        }
    }

    Ok(())
}
