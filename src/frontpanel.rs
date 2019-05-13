use std::net::TcpStream;
use std::io::Write;

pub fn set_relay(status: bool) -> Result<(), std::io::Error> {
    let mut stream = TcpStream::connect("127.0.0.1:1450")?;

    if status {
        stream.write(b"relay on\r\n");
    } else {
        stream.write(b"relay off\r\n");
    }

    Ok(())
}