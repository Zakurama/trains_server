use std::{collections::VecDeque, net::UdpSocket};
use cantons::*;

fn main() {
    let mut trains: VecDeque<Train> = VecDeque::new();
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <ip address> <port>", args[0]);
        std::process::exit(1);
    }
    let ip_address = &args[1];
    let port = &args[2];
    let socket_address = format!("{}:{}", ip_address, port);
    let socket = UdpSocket::bind(socket_address).expect("Could not bind socket");
    println!("Waiting for clients to connect...");
    loop {
        handle_client(&socket, &mut trains);
    }
}
