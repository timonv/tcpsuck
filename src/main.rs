// extern crate tcpsuck;

use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::{BufferedStream, BufferedWriter};
use std::sync::{RWLock, Arc};
use std::collections::HashMap;

// Has to be a better way to do this?
use chat_server::ChatServer;
mod chat_server;

// TODO
// * Refactor To 'Client' structs
// * Clean up unwraps
// * Experiment with a single dispatch flow setup:
// Client -> Dispatcher -> Server
//  ^-----------------------|
// * Cleanup to files
// * Tests :')
// * Status displays
// * Better UI
// * Commands
// * Timestamps
// * Convert over usage of Strings to slices

// MAYBE
// * Encryption
// * Distributed setup



fn main() {
    let listener = TcpListener::bind("127.0.0.1:6667");
    let mut acceptor = listener.listen();

    // Used for passing chat messages between threads
    let (tx, rx) = channel::<(String, String)>();
    let server = ChatServer::<TcpStream>::new();

    server.listen_and_broadcast(rx);

    for stream in acceptor.incoming() {
        let txc = tx.clone();
        let server_copy = server.clone();

        // We can refactor this to channels, or encapsulate the clone, abstracting over
        // concurrency, or both.
        spawn(move || {
            handle_client(server_copy, stream.unwrap(), txc)
        })
    }
}

fn handle_client(server: ChatServer<TcpStream>, stream: TcpStream, tx: Sender<(String, String)>) {
    let mut copy = stream.clone();
    let mut stream = BufferedStream::new(stream);
    let name = get_name(&mut stream);

    server.register_user(name.clone(), copy);

    for line in stream.lines() {
        let data = (name.clone(), line.unwrap());
        tx.send(data);
    }
}

fn get_name(stream: &mut BufferedStream<TcpStream>) -> String {
    stream.write_str("Name: ".as_slice()).unwrap();
    stream.flush().unwrap();
    stream.read_line().unwrap().trim().to_string()
}
