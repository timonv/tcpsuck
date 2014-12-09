use std::io::{TcpStream, BufferedWriter};
use std::sync::{RWLock, Arc};
use std::collections::HashMap;

type SharedUserStreamMap = Arc<RWLock<HashMap<String, TcpStream>>>;

pub struct ChatServer {
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
