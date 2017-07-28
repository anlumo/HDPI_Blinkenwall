use glium;
use glium::Surface;
use glium::index::PrimitiveType;
use glium::backend::glutin_backend::GlutinFacade;
use glium::texture::texture2d::Texture2d;
use glium::texture::{RawImage2d, ClientFormat, MipmapsOption, UncompressedFloatFormat};
use std::net::{TcpListener, TcpStream};
use bdf;
use std::io::{Read, ErrorKind};
use std::collections::HashSet;

pub struct Poetry {
    listener: TcpListener,
    speed: u32,
    font: bdf::Font,
    incoming: Vec<(TcpStream, Vec<u8>)>,
}

impl Poetry {
    pub fn new(display: &GlutinFacade, ip: &str, port: u16, font: &str, speed: u32) -> Poetry {
        let addr = format!("{}:{}", ip, port);
        info!("Poetry listening on {}...", addr);
        let font = bdf::open(font).expect("Cannot load font file");

        let listener = TcpListener::bind(addr).unwrap();
        listener.set_nonblocking(true).expect("Cannot set nonblocking mode.");

        Poetry {
            listener: listener,
            speed: speed,
            font: font,
            incoming: Vec::new(),
        }
    }

    pub fn step(&mut self) {
        while let Ok((stream, addr)) = self.listener.accept() {
            info!("Poetry connection from {}", addr);
            stream.set_nonblocking(true).expect("Cannot set nonblocking mode.");
            self.incoming.push((stream, Vec::new()));
        }

        for i in (0..self.incoming.len()).rev() {
            let mut drop_connection = false;
            {
                let &mut (ref mut stream, ref mut buffer) = &mut self.incoming[i];
                let mut readbuffer = [0_u8; 1024];
                loop {
                    match stream.read(&mut readbuffer) {
                        Ok(count) => {
                            if count == 0 {
                                if let Ok(s) = String::from_utf8(buffer.clone()) {
                                    info!("Got string {}", &s);
                                }
                                info!("Poetry connection to {} closed.", stream.peer_addr().unwrap());

                                drop_connection = true;
                                break;
                            }
                            buffer.append(&mut readbuffer[0..count].to_vec());
                        },
                        Err(err) => {
                            match err.kind() {
                                ErrorKind::Interrupted => {
                                    continue;
                                },
                                ErrorKind::WouldBlock => {
                                    break;
                                },
                                _ => {
                                    error!("Poetry connection to {} got error {}.", stream.peer_addr().unwrap(), err);
                                    drop_connection = true;
                                },
                            }
                        }
                    }
                }
            }

            if drop_connection {
                self.incoming.swap_remove(i);
            }
        }
    }
}
