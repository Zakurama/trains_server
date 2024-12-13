use std::{collections::VecDeque, net::UdpSocket};

const TGVSPEED: u8 = 20;
const TERSPEED: u8 = 10;
const INTSPEED: u8 = 5;
const MAXTRAINS: u8 = 100;

pub struct Train{
    location: u8,
    name: String,
    eoa: u8,
    speed: u8,
}

impl Train{
    fn new(name: String, eoa: u8) -> Train{
        let speed:u8;
        match &name[0..3]{
            "TGV" => {
                speed = TGVSPEED
            }
            "TER" => {
                speed = TERSPEED
            }
            "INT" => {
                speed = INTSPEED
            }
            _ => {
                speed = INTSPEED
            }
        }
        Train{
            location: 0,
            name,
            eoa,
            speed,
        }
    }

    fn forward(&mut self, block: u8) -> String {
        if block > self.eoa {
            return String::from("Block is greater than EOA");
        }
        else if block <= self.location {
            return String::from("Block is less than or equal to current location");
        }
        else if block > self.location + self.speed {
            return String::from("Block is unreachable");
        }
        else {
            self.location = block;
            return String::from("ACK");
        }
    }
}

fn find(train_list: &VecDeque<Train>, name: &String) -> Option<u8> {
    for (index, train) in train_list.iter().enumerate() {
        if &train.name == name {
            return Some(index as u8);
        }
    }
    return None;
}

fn parse_message(message: &String) -> (String, String, u8) {
    let parts: Vec<&str> = message.split(":").collect();
    match parts.len() {
        2 => {
            return (String::from(parts[0]), String::from(parts[1]), 0);
        }
        3 => {
            return (String::from(parts[0]), String::from(parts[1]), parts[2].parse::<u8>().unwrap_or(0));
        }
        _ => {
            return (String::from(""), String::from(""), 0);
        }
    }
}

fn handle_register(train_list: &mut VecDeque<Train>, name: String, eoa: u8) -> String {
    if find(&train_list, &name).is_some() {
        return String::from("Already registered");
    }
    else if !train_list.is_empty() && train_list[train_list.len() - 1].location == 0 {
        return String::from("First block occupied");
    }
    else{
        let train = Train::new(name, eoa);
        train_list.push_back(train);
        return String::from("ACK");
    }
}

fn unregister(train_list: &mut VecDeque<Train>, name: String) -> String {
    match find(&train_list, &name) {
        Some(index) => {
            if train_list[index as usize].location != MAXTRAINS {
                return String::from("Train is not in the last block");
            }
            else {
                train_list.pop_front();
                return String::from("ACK");
            }
        }
        None => {
            return String::from("Not registered");
        }
    }
}

fn handle_message(message: &String, trains: &mut VecDeque<Train>) -> String {
    let (name, command, block) = parse_message(message);
    let response: String;
    match command.as_str() {
        "reg" => {
            response = handle_register(trains, name, block);
        }
        "unr" => {
            response = unregister(trains, name);
        }
        "for" => {
            match find(&trains, &name) {
                Some(index) => {
                    response = trains[index as usize].forward(block);
                }
                None => {
                    response = String::from("Not registered");
                }
            }
        }
        _ => {
            response = String::from("Invalid command");
        }
    }
    return response;
}

pub fn handle_client(socket: &UdpSocket, trains: &mut VecDeque<Train>) {
    let mut buf = [0; 1024];
    let (amt, src) = socket.recv_from(&mut buf).unwrap();
    let message = String::from_utf8_lossy(&buf[..amt]).to_string();
    println!("Received message: {}", message);
    let response = handle_message(&message, trains);
    println!("Sent message: {}", response);
    socket.send_to(response.as_bytes(), &src).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tgv_train_creation() {
        let train = Train::new(String::from("TGV123"), 10);
        assert_eq!(train.name, "TGV123");
        assert_eq!(train.eoa, 10);
        assert_eq!(train.speed, TGVSPEED);
    }

    #[test]
    fn test_ter_train_creation() {
        let train = Train::new(String::from("TER456"), 20);
        assert_eq!(train.name, "TER456");
        assert_eq!(train.eoa, 20);
        assert_eq!(train.speed, TERSPEED);
    }

    #[test]
    fn test_int_train_creation() {
        let train = Train::new(String::from("INT789"), 30);
        assert_eq!(train.name, "INT789");
        assert_eq!(train.eoa, 30);
        assert_eq!(train.speed, INTSPEED);
    }

    #[test]
    fn test_default_train_creation() {
        let train = Train::new(String::from("ABC123"), 40);
        assert_eq!(train.name, "ABC123");
        assert_eq!(train.eoa, 40);
        assert_eq!(train.speed, INTSPEED);
    }

    #[test]
    fn test_forward_success() {
        let mut train = Train::new(String::from("TGV123"), 10);
        assert_eq!(train.forward(5), String::from("ACK"));
        assert_eq!(train.location, 5);
    }

    #[test]
    fn test_forward_block_greater_than_eoa() {
        let mut train = Train::new(String::from("TER456"), 2);
        assert_eq!(train.forward(5), String::from("Block is greater than EOA"));
        assert_eq!(train.location, 0);
    }

    #[test]
    fn test_forward_block_less_than_or_equal_to_current_location() {
        let mut train = Train::new(String::from("INT789"), 30);
        train.location = 10;
        assert_eq!(train.forward(9), String::from("Block is less than or equal to current location"));
        assert_eq!(train.location, 10);
    }

    #[test]
    fn test_forward_block_unreachable() {
        let mut train = Train::new(String::from("TGV123"), 100);
        assert_eq!(train.forward(25), String::from("Block is unreachable"));
        assert_eq!(train.location, 0);
    }

    #[test]
    fn test_parse_message() {
        let message = String::from("TGV123:forward:5");
        let (name, command, block) = parse_message(&message);
        assert_eq!(name, "TGV123");
        assert_eq!(command, "forward");
        assert_eq!(block, 5);
    }

    #[test]
    fn test_parse_message_invalid_block() {
        let message = String::from("TER456:forward:abc");
        let (name, command, block) = parse_message(&message);
        assert_eq!(name, "TER456");
        assert_eq!(command, "forward");
        assert_eq!(block, 0);
    }

    #[test]
    fn test_handle_register_success() {
        let mut train_list = VecDeque::new();
        let response = handle_register(&mut train_list, String::from("TGV123"), 10);
        assert_eq!(response, "ACK");
        assert_eq!(train_list.len(), 1);
        assert_eq!(train_list[0].name, "TGV123");
    }

    #[test]
    fn test_handle_register_already_registered() {
        let mut train_list = VecDeque::new();
        train_list.push_back(Train::new(String::from("TGV123"), 10));
        let response = handle_register(&mut train_list, String::from("TGV123"), 10);
        assert_eq!(response, "Already registered");
    }

    #[test]
    fn test_handle_register_first_block_occupied() {
        let mut train_list = VecDeque::new();
        train_list.push_back(Train::new(String::from("TGV123"), 10));
        train_list[0].location = 0;
        let response = handle_register(&mut train_list, String::from("TER456"), 20);
        assert_eq!(response, "First block occupied");
    }

    #[test]
    fn test_unregister_success() {
        let mut train_list = VecDeque::new();
        let mut train = Train::new(String::from("TGV123"), 10);
        train.location = 100;
        train_list.push_back(train);
        let response = unregister(&mut train_list, String::from("TGV123"));
        assert_eq!(response, "ACK");
        assert!(train_list.is_empty());
    }

    #[test]
    fn test_unregister_not_registered() {
        let mut train_list = VecDeque::new();
        let response = unregister(&mut train_list, String::from("TGV123"));
        assert_eq!(response, "Not registered");
    }

    #[test]
    fn test_unregister_not_in_last_block() {
        let mut train_list = VecDeque::new();
        let train = Train::new(String::from("TGV123"), 10);
        train_list.push_back(train);
        let response = unregister(&mut train_list, String::from("TGV123"));
        assert_eq!(response, "Train is not in the last block");
        assert_eq!(train_list.len(), 1);
    }
    
    #[test]
    fn test_handle_message_register() {
        let mut train_list = VecDeque::new();
        let message = String::from("TGV123:reg:10");
        let response = handle_message(&message, &mut train_list);
        assert_eq!(response, "ACK");
        assert_eq!(train_list.len(), 1);
        assert_eq!(train_list[0].name, "TGV123");
    }

    #[test]
    fn test_handle_message_unregister() {
        let mut train_list = VecDeque::new();
        let mut train = Train::new(String::from("TGV123"), 10);
        train.location = 100;
        train_list.push_back(train);
        let message = String::from("TGV123:unr:0");
        let response = handle_message(&message, &mut train_list);
        assert_eq!(response, "ACK");
        assert!(train_list.is_empty());
    }

    #[test]
    fn test_handle_message_forward() {
        let mut train_list = VecDeque::new();
        let train = Train::new(String::from("TGV123"), 10);
        train_list.push_back(train);
        let message = String::from("TGV123:for:5");
        let response = handle_message(&message, &mut train_list);
        assert_eq!(response, "ACK");
        assert_eq!(train_list[0].location, 5);
    }

    #[test]
    fn test_handle_message_invalid_command() {
        let mut train_list = VecDeque::new();
        let message = String::from("TGV123:invalid:5");
        let response = handle_message(&message, &mut train_list);
        assert_eq!(response, "Invalid command");
    }

    #[test]
    fn test_handle_message_not_registered() {
        let mut train_list = VecDeque::new();
        let message = String::from("TGV123:for:5");
        let response = handle_message(&message, &mut train_list);
        assert_eq!(response, "Not registered");
    }

    #[test]
    fn test_handle_client_register() {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = socket.local_addr().unwrap();
        let mut train_list = VecDeque::new();
        let message = String::from("TGV123:reg");
        socket.send_to(message.as_bytes(), addr).unwrap();
        handle_client(&socket, &mut train_list);
        assert_eq!(train_list.len(), 1);
        assert_eq!(train_list[0].name, "TGV123");
    }

    #[test]
    fn test_handle_client_unregister() {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = socket.local_addr().unwrap();
        let mut train = Train::new(String::from("TGV123"), 10);
        train.location = 100;
        let mut train_list = VecDeque::new();
        train_list.push_back(train);
        let message = String::from("TGV123:unr");
        socket.send_to(message.as_bytes(), addr).unwrap();
        handle_client(&socket, &mut train_list);
        assert!(train_list.is_empty());
    }

    #[test]
    fn test_handle_client_forward() {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = socket.local_addr().unwrap();
        let mut train_list = VecDeque::new();
        let train = Train::new(String::from("TGV123"), 10);
        train_list.push_back(train);
        let message = String::from("TGV123:for:5");
        socket.send_to(message.as_bytes(), addr).unwrap();
        handle_client(&socket, &mut train_list);
        assert_eq!(train_list[0].location, 5);
    }

    #[test]
    fn test_handle_client_invalid_command() {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = socket.local_addr().unwrap();
        let mut train_list = VecDeque::new();
        let message = String::from("TGV123:invalid");
        socket.send_to(message.as_bytes(), addr).unwrap();
        handle_client(&socket, &mut train_list);
        assert!(train_list.is_empty());
    }

    #[test]
    fn test_handle_client_not_registered() {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = socket.local_addr().unwrap();
        let mut train_list = VecDeque::new();
        let message = String::from("TGV123:for:5");
        socket.send_to(message.as_bytes(), addr).unwrap();
        handle_client(&socket, &mut train_list);
        assert!(train_list.is_empty());
    }

}
