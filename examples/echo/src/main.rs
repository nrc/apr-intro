#![feature(await_macro, async_await, futures_api)]

use std::net::SocketAddr;

// tokio is a common foundation for async networking
use tokio::await;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

// Read data from `stream` and echo it back.
async fn handle(mut stream: TcpStream) {
    let mut buf = [0; 1024];

    loop {
        match await!(stream.read_async(&mut buf)).unwrap() {
            0 => break, // Socket closed.
            n => {
                // Send the data back.
                await!(stream.write_all_async(&buf[0..n])).unwrap();
            }
        }
    }
}

// Listen for incoming TCP data.
async fn listen(addr: SocketAddr) {
    let listener = TcpListener::bind(&addr).unwrap();
    let mut incoming = listener.incoming();

    while let Some(stream) = await!(incoming.next()) {
        let stream = stream.unwrap();
        tokio::spawn_async(handle(stream));
    }
}

fn main() {
    tokio::run_async(listen("127.0.0.1:8080".parse().unwrap()));
}
