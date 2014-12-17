// Has to be a better way to do this?
use server::Server;
use broadcaster::Broadcaster;
mod server;

mod broadcaster;
// TODO
// * Refactor To 'Client' structs
// * Clean up unwraps
// * Handle sigterm
// * Experiment with a single dispatch flow setup:
// Client -> Dispatcher -> Server
//  ^-----------------------|
// * Status displays
// * Better UI
// * Commands
// * Timestamps
// * Convert over usage of Strings to slices

// MAYBE
// * Encryption
// * Distributed setup


#[allow(dead_code)]
fn main() {
    let server = Server::new();
    server.start("127.0.0.1:6667");
}
