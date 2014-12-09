use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::{BufferedStream, BufferedWriter};
use std::sync::{RWLock, Arc};
use std::collections::HashMap;


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

// MAYBE
// * Encryption
// * Distributed setup

type SharedUserStreamMap = Arc<RWLock<HashMap<String, TcpStream>>>;

struct ChatServer {
    users: SharedUserStreamMap
}

impl ChatServer {
    pub fn new() -> ChatServer {
        ChatServer { users: Arc::new(RWLock::new(HashMap::<String, TcpStream>::new())) }
    }

    pub fn register_user(&self, name: String, stream: TcpStream) {
        let mut streams = self.users.write();
        streams.insert(name, stream);
        streams.downgrade();
    }

    pub fn listen_and_broadcast(&self, rec: Receiver<(String, String)>) {
        let copy = self.clone();
        spawn(proc() {
            loop {
                let (user, message) = rec.recv();
                copy.broadcast(user, message);
            }
        })
    }

    fn broadcast(&self, origin: String, message: String) {
        let message = self.format_message(&origin, &message);
        let users = self.users.clone();
        let unlocked = users.read();

        for (user, stream) in unlocked.iter() {
            if *user != origin { // Why?
                let mut writer = BufferedWriter::new(stream.clone());
                writer.write_line(message.as_slice()).unwrap();
                writer.flush().unwrap();
            }
        }
    }

    fn format_message(&self, name: &String, message: &String) -> String {
        format!("[{}]: {}", name, message.trim())
    }
}

// TODO Remove need for this by doing everything over spawns with channels
impl Clone for ChatServer {
    fn clone(&self) -> ChatServer {
        ChatServer { users: self.users.clone() }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6667");
    let mut acceptor = listener.listen();

    // Used for passing chat messages between threads
    let (tx, rx) = channel::<(String, String)>();
    let server = ChatServer::new();

    server.listen_and_broadcast(rx);

    for stream in acceptor.incoming() {
        let txc = tx.clone();
        let server_copy = server.clone();

        // We can refactor this to channels, or encapsulate the clone, abstracting over
        // concurrency, or both.
        spawn(proc() {
            handle_client(server_copy, stream.unwrap(), txc)
        })
    }
}

fn handle_client(server: ChatServer, stream: TcpStream, tx: Sender<(String, String)>) {
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
