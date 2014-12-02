use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::{BufferedStream, BufferedWriter};
use std::sync::{RWLock, Arc};


// TODO
// * Dont see own chat messages
// * Refactor to ideomatic rust
// * Timestamps
// * Print list of connected clients to each

// MAYBE
// * Encryption
// * Distributed setup

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6667");
    let mut acceptor = listener.listen();

    // Used for passing chat messages between threads
    let (tx, rx) = channel::<String>();

    // Vec needs typehint otherwise `cannot move out of dereference of `&`-pointer` errors
    let streams_lock = Arc::new(RWLock::new(Vec::<TcpStream>::new()));
    let streams_lock2 = streams_lock.clone(); // Clone before spawn

    spawn(proc() {
        loop {
            let message: String = rx.recv(); //blocks
            let streams = streams_lock2.read();

            for stream in streams.iter() {
                let mut writer = BufferedWriter::new(stream.clone());
                writer.write_line(message.as_slice()).unwrap();
                writer.flush().unwrap();
                // Do loops clear scope per iter?
            }
        }
    });

    for stream in acceptor.incoming() {
        match stream {
            Err(e) => { println!("Error {}", e); }
            Ok(stream) => {
                let mut streams = streams_lock.write();
                streams.push(stream.clone());
                streams.downgrade();


                // Clone before spawn
                let txc = tx.clone();
                spawn(proc() {
                    handle_client(stream, txc);
                });
            }
        }
    }
}

fn handle_client(stream: TcpStream, tx: Sender<String>) {
    let mut stream = BufferedStream::new(stream);
    let name = get_name(&mut stream);

    stream.write_line(format!("Your name is {}", name).as_slice()).unwrap();
    stream.flush().unwrap();

    for line in stream.lines() {
        // TODO Error handling + connection closing
        let line = line.unwrap();
        let message = format!("[{}]: {}", name, line.trim());
        tx.send(message);
    }
}

fn get_name(stream: &mut BufferedStream<TcpStream>) -> String {
    stream.write_str("Name: ".as_slice()).unwrap();
    stream.flush().unwrap();
    stream.read_line().unwrap().trim().to_string()
}
