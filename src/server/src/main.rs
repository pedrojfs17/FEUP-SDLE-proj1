use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
// use serde_json;
// use std::fs::File;
// use std::thread;
// use std::time;

fn main() {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::ROUTER).unwrap();
    socket.set_router_mandatory(true).unwrap();

    socket
        .bind("tcp://*:5559")
        .expect("failed binding socket");

    let mut topics: HashMap<String, HashMap<String, VecDeque<String>>> = HashMap::new();
    let mut pending_requests: HashMap<String, HashSet<String>> = HashMap::new();

    // let mut file = File::create("state");

    loop {
        let (request_id, request) = read_message(&socket);

        parse_request(&request_id, request, &socket, &mut topics, &mut pending_requests);

        for (topic, map) in &topics {
            println!("Topic: {}", topic);
            for (id, deq) in map {
                println!("\t{} : {:?}", id, deq);
            }
        }
    }
}

// fn routine_save(file: &File, topics: &HashMap<String, HashMap<String, VecDeque<String>>>) {
//     serde_json::to_writer(file, topics);
//  }

fn read_message(socket: &zmq::Socket) -> (String, String) {
    let request_id = socket
        .recv_string(0)
        .expect("Failed receiving id")
        .unwrap();

    // Empty Packet
    assert!(socket.recv_string(0).expect("Failed receiving empty").unwrap() == "");

    let request = socket
        .recv_string(0)
        .expect("Failed receiving request")
        .unwrap();

    println!("Recieved request from: {} -> {}", request_id, request);

    return (request_id, request);
}

fn send_message(socket: &zmq::Socket, id: &String, message: &str) -> i32 {
    let id_part_result = socket.send(id.as_bytes(), zmq::SNDMORE);
    match id_part_result {
        Ok(v) => println!("Result OK: {:?}", v),
        Err(e) => {
            println!("Result Error: {:?}", e);
            return -1;
        },
    }

    let empty_part_result = socket.send("", zmq::SNDMORE);
    match empty_part_result {
        Ok(v) => println!("Result OK: {:?}", v),
        Err(e) => {
            println!("Result Error: {:?}", e);
            return -1;
        },
    }

    let message_result = socket.send(message, 0);
    match message_result {
        Ok(v) => println!("Result OK: {:?}", v),
        Err(e) => {
            println!("Result Error: {:?}", e);
            return -1;
        },
    }

    return 0;
}

fn parse_request(request_id: &String, request: String, socket: &zmq::Socket, topics: &mut HashMap<String, HashMap<String, VecDeque<String>>>, pending_requests: &mut HashMap<String, HashSet<String>>) {
    let split: Vec<_> = request.splitn(2, " ").collect();

    let action = split[0];

    let start_bytes = split[1].find("[").unwrap_or(0) + 1;
    let end_bytes = split[1].find("]").unwrap_or(split[1].len());
    let topic = &split[1][start_bytes..end_bytes];

    match action {
        "SUB" => {
            // Check if topic already exists
            if topics.contains_key(topic) {
                let topic_map = topics.get_mut(topic).unwrap();

                // Check if topic does not have the susbcriber
                if !topic_map.contains_key(request_id) {
                    topic_map.insert(String::from(request_id), VecDeque::new());
                }
            } else {
                let mut topic_map: HashMap<String, VecDeque<String>> = HashMap::new();
                topic_map.insert(String::from(request_id), VecDeque::new());

                topics.insert(topic.to_string(), topic_map);
            }

            send_message(&socket, &request_id, "OK");
        },

        "UNSUB" => {
            // Check if topic exists
            if topics.contains_key(topic) {
                let topic_map = topics.get_mut(topic).unwrap();
                topic_map.remove(request_id);
            }
            
            send_message(&socket, &request_id, "OK");
        },

        "GET" => {
            // Check if topic already exists
            if topics.contains_key(topic) {
                let topic_map = topics.get_mut(topic).unwrap();

                // Check if topic does not have the susbcriber
                if topic_map.contains_key(request_id) {
                    let value = topic_map.get_mut(request_id).unwrap().pop_front();
                    if value == None {
                        add_pending_request(pending_requests, String::from(topic), String::from(request_id));
                    } else {
                        send_message(&socket, &request_id, value.unwrap().as_str());
                    }
                } else { //Not Subscribed
                    send_message(&socket, &request_id, "NS");
                }
            } else { //Not Found
                send_message(&socket, &request_id, "NF");
            }
        },

        "PUT" => {
            if topics.contains_key(topic) {
                let topic_map = topics.get_mut(topic).unwrap();
                let msg = &split[1][end_bytes + 1..];

                for (_, queue) in topic_map {
                    queue.push_back(String::from(msg.trim()));
                }

            } else {
                let topic_map: HashMap<String, VecDeque<String>> = HashMap::new();
                topics.insert(topic.to_string(), topic_map);
            }

            send_message(&socket, &request_id, "OK");

            check_pending_requests(String::from(topic), socket, topics, pending_requests);
        },

        _ => {
            send_message(&socket, &request_id, "NOK");
        },
    }

}

fn add_pending_request(map: &mut HashMap<String, HashSet<String>>, topic: String, id: String) {
    if map.contains_key(&topic) {
        map.get_mut(&topic).unwrap().insert(id);
    } else {
        let mut set: HashSet<String> = HashSet::new();
        set.insert(id);
        map.insert(topic, set);
    }
}

fn check_pending_requests(topic: String, socket: &zmq::Socket, topics: &mut HashMap<String, HashMap<String, VecDeque<String>>>, pending_requests: &mut HashMap<String, HashSet<String>>) {
    if pending_requests.contains_key(&topic) {
        let requests = pending_requests.get_mut(&topic).unwrap();
        let topic_map = topics.get_mut(&topic).unwrap();

        for id in requests.iter() {
            let value = topic_map.get_mut(id).unwrap().pop_front();
            send_message(&socket, &id, value.unwrap().as_str());
        }

        pending_requests.remove(&topic);
    }
}

