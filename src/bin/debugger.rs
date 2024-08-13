use std::{
    io::{stdin, BufRead, BufReader, Write},
    net::TcpStream,
};

fn main() -> Result<(), String> {
    let stream = TcpStream::connect("127.0.0.1:3000").unwrap();
    let mut reader = BufReader::new(&stream);
    for line in stdin().lock().lines() {
        let line = line.map_err(|error| error.to_string())?;
        writeln!(&stream, "{}", line).unwrap();
        let mut response = String::new();
        reader.read_line(&mut response).unwrap();
        println!("response: {}", response);
    }

    Ok(())
}
