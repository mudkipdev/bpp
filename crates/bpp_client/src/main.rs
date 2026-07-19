pub mod assets;
pub mod client;
pub mod glfw_context;
pub mod inputs;
pub mod renderers;
pub mod window;

use client::Client;

fn main() {
    let mut client = Client::new();
    client.run();
}
