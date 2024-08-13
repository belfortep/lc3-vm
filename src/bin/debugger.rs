use std::{
    io::{stdin, BufRead, BufReader, Write},
    net::TcpStream,
};

fn main() -> Result<(), String> {
    let stream = TcpStream::connect("127.0.0.1:3000").map_err(|error| error.to_string())?;
    let mut reader = BufReader::new(&stream);
    for line in stdin().lock().lines() {
        let line = line.map_err(|error| error.to_string())?;
        writeln!(&stream, "{}", line).map_err(|error| error.to_string())?;
        let mut response = String::new();
        reader
            .read_line(&mut response)
            .map_err(|error| error.to_string())?;
        println!("response: {}", response);
    }

    Ok(())
}
