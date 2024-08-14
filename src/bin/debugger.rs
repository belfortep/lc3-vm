use std::{
    io::{stdin, BufRead, BufReader, Write},
    net::TcpStream,
};

fn print_instructions() {
    println!("Instructions: ");
    println!("<n> to execute one instruction");
    println!("<a number> to execute that number of instructions (recommended)");
    println!("<r> to print the state of the registers of the program");
    println!("remember to use the program you are debugging if it needs user input");
}

fn main() -> Result<(), String> {
    let stream = TcpStream::connect("127.0.0.1:3000").map_err(|error| error.to_string())?;
    print_instructions();
    let mut reader = BufReader::new(&stream);
    for line in stdin().lock().lines() {
        let line = line.map_err(|error| error.to_string())?;
        writeln!(&stream, "{}", line).map_err(|error| error.to_string())?;
        let mut response = String::new();
        reader
            .read_line(&mut response)
            .map_err(|error| error.to_string())?;
        let response = response.split("::");
        for data in response {
            println!("{}", data);
        }
    }

    Ok(())
}
