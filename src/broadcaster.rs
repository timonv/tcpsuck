use std::io::BufferedWriter;
use std::sync::{RWLock, Arc};
use std::collections::HashMap;

pub struct Broadcaster<T: 'static + Reader + Writer> {
    users: Arc<RWLock<HashMap<String, T>>>
}

// Generics are kinda viral...
impl<T: Reader + Writer + Clone> Broadcaster<T> {
    pub fn new() -> Broadcaster<T> {
        Broadcaster { users: Arc::new(RWLock::new(HashMap::<String, T>::new())) }
    }

    pub fn register_user(&self, name: String, stream: T) {
        let mut streams = self.users.write();
        streams.insert(name, stream);
    }

    pub fn is_registered(&self, name: &str) -> bool {
        // TODO Check if stream is still valid
        let streams = self.users.read();
        streams.contains_key(name)
    }

    // RFC Maybe better to return the senders of the channel here?
    pub fn listen_and_broadcast(&self, rec: Receiver<(String, String)>) {
        let clone = self.clone();
        spawn(move || {
            loop {
                match rec.recv_opt() {
                    Ok( (user, message) ) => clone.broadcast(user, message),
                    Err(_) => return // Obviously problematic if a client disconnects
                }
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
impl<T: Reader + Writer>Clone for Broadcaster<T> {
    fn clone(&self) -> Broadcaster<T> {
        Broadcaster { users: self.users.clone() }
    }
}

#[cfg(test)]
mod test {
    use super::Broadcaster;
    use std::io::IoResult;
    use std::sync::{RWLock, Arc};
    use std::str;
    use std::io::timer::sleep;
    use std::time::duration::Duration;

    #[test]
    fn test_registering_of_user() {
        let server = Broadcaster::new();
        let stream = FakeStream::new();

        server.register_user("Pietje".to_string(), stream);

        assert!(server.is_registered("Pietje"))
    }

    #[test]
    fn test_broadcasting_a_message() {
        let server = Broadcaster::new();
        let pietje = FakeStream::new();

        server.register_user("Pietje".to_string(), pietje.clone());
        let message = "Baby don't hurt me, don't hurt me, no more!";

        let (tx, rx) = channel::<(String, String)>();
        server.listen_and_broadcast(rx);
        tx.send(("Jantje".to_string(), message.to_string()));

        let expected = "[Jantje]: ".to_string() + message.to_string();

        // Should be possible with futures
        timeout(5, || {
            pietje.read_output() != ""
        });

        assert_eq!(expected.as_slice(), pietje.read_output().trim());
    }

    struct FakeStream {
        input: String,
        output: Arc<RWLock<String>>
    }

    impl FakeStream {
        pub fn new() -> FakeStream {
            // Because when we clone, we it to reference the same output stream (as a TcpConnection
            // would)
            FakeStream { input: String::new(), output: Arc::new(RWLock::new(String::new()))}
        }

        pub fn read_output(&self) -> String {
            self.output.clone().read().clone()
        }
    }

    impl Reader for FakeStream {
        fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
            let mut len = 0;
            // THIS IS GENIUS
            for (slot, byte) in buf.iter_mut().zip(self.input.as_bytes().iter())  {
                *slot = *byte;
                len += 1;
            }

            Ok(len)
        }

    }

    impl Writer for FakeStream {
        fn write(&mut self, buf: &[u8]) -> IoResult<()> {
            let mut unlocked = self.output.write();
            *unlocked = str::from_utf8(buf).expect("Test your shizzle, pizzle").to_string();
            Ok(())
        }
    }

    impl Clone for FakeStream {
        fn clone(&self) -> FakeStream {
            FakeStream { input: self.input.clone(), output: self.output.clone() }
        }
    }

    fn timeout(max: int, f: || -> bool) -> bool {
        for _ in range(0, max) {
            if f() == true {
                return true
            }
            sleep(Duration::milliseconds(100));
        }
        panic!("TIMOUT TIMOUT")
    }
}
