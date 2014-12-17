use std::io::{TcpListener, TcpStream, Acceptor, Listener, BufferedStream};
use super::broadcaster::Broadcaster;

pub struct Server;

impl Server {
    pub fn new() -> Server {
        Server
    }

    pub fn start(&self, addr: &str) {
        let listener = TcpListener::bind(addr);
        let mut acceptor = listener.listen();

        // Used for passing chat messages between threads
        let (tx, rx) = channel::<(String, String)>();
        let server = Broadcaster::<TcpStream>::new();

        server.listen_and_broadcast(rx);

        for stream in acceptor.incoming() {
            let txc = tx.clone();
            let server_copy = server.clone();

            // We can refactor this to channels, or encapsulate the clone, abstracting over
            // concurrency, or both.

            // THIS IS WRONG
            spawn(move || {
                Server::handle_client(server_copy, stream.unwrap(), txc)
            })
        }
    }

    // TODO Borrow!
    fn handle_client(server: Broadcaster<TcpStream>, stream: TcpStream, tx: Sender<(String, String)>) {
        // too many clones here
        let copy = stream.clone();
        let mut stream = BufferedStream::new(stream);
        let name = Server::get_name(&mut stream);

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
}

#[cfg(test)]
mod test {
//todo
}
