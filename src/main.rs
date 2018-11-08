extern crate crypto;
extern crate md5;

use crypto::rc4::Rc4;
use std::error::Error;
use std::io::prelude::*;
use std::net::Shutdown;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

use crypto::symmetriccipher::SynchronousStreamCipher;

static C_LISTEN: &str = "0.0.0.0:51958";
static C_CIPHER: &str = "daze";

// fn read_exact(stream: &mut TcpStream, c: &mut Rc4, buf: &mut [u8]) -> Result<(), Box<Error>> {
//     let mut src = vec![0; buf.len()];
//     stream.read_exact(&mut src)?;
//     c.process(&src, buf);
//     Ok(())
// }

fn read(stream: &mut TcpStream, c: &mut Rc4, buf: &mut [u8]) -> Result<usize, Box<Error>> {
    let mut src = vec![0; buf.len()];
    let n = stream.read(&mut src)?;
    c.process(&src[..n], &mut buf[..n]);
    Ok(n)
}

fn daze(mut src_stream: TcpStream) {
    let mut buf: Vec<u8> = vec![0; 128];
    src_stream.read(&mut buf).unwrap();
    let mut raw: Vec<u8> = md5::compute(C_CIPHER).0.iter().cloned().collect();
    buf.append(&mut raw);
    let mut cipher_a = Rc4::new(&buf);
    let mut cipher_b = Rc4::new(&buf);
    let mut buf: Vec<u8> = vec![0; 12];
    if let Err(err) = read(&mut src_stream, &mut cipher_a, &mut buf) {
        println!("{}", err);
        return;
    };
    let mut buf: Vec<u8> = vec![0; buf[11] as usize];
    if let Err(err) = read(&mut src_stream, &mut cipher_a, &mut buf) {
        println!("{}", err);
        return;
    };
    let dst = String::from_utf8(buf).unwrap();
    println!("Connect {}", dst);

    let mut dst_stream = TcpStream::connect(&dst).unwrap();
    let mut src_stream_copy = src_stream.try_clone().unwrap();
    let mut dst_stream_copy = dst_stream.try_clone().unwrap();
    thread::spawn(move || {
        let mut buf: Vec<u8> = vec![0; 32 * 1024];
        loop {
            let n = match read(&mut src_stream_copy, &mut cipher_a, &mut buf) {
                Ok(data) => data,
                _ => 0,
            };
            if n == 0 {
                break;
            };
            if dst_stream_copy.write_all(&buf[..n]).is_err() {
                break;
            };
        }
        src_stream_copy.shutdown(Shutdown::Both).unwrap();
        dst_stream_copy.shutdown(Shutdown::Both).unwrap();
    });

    let mut buf: Vec<u8> = vec![0; 32 * 1024];
    loop {
        let n = match read(&mut dst_stream, &mut cipher_b, &mut buf) {
            Ok(data) => data,
            _ => 0,
        };
        if n == 0 {
            break;
        };
        if src_stream.write_all(&buf[..n]).is_err() {
            break;
        };
    }
    dst_stream.shutdown(Shutdown::Both).unwrap();
    src_stream.shutdown(Shutdown::Both).unwrap();
}

fn main() {
    let listener = TcpListener::bind(C_LISTEN).unwrap();
    println!("Listen and server on {}", C_CIPHER);
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        thread::spawn(move || daze(stream));
    }
}
