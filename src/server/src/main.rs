use std::collections::HashMap;
use std::collections::VecDeque;

fn main() {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::ROUTER).unwrap();
    socket.set_router_mandatory(true).unwrap();

    socket
        .bind("tcp://*:5559")
        .expect("failed binding socket");

    let mut topics: HashMap<String, HashMap<String, VecDeque<String>>> = HashMap::new();

    loop {
        let (request_id, request) = read_message(&socket);

        parse_request(&request_id, request, &socket, &mut topics);

        for (topic, map) in &topics {
            println!("Topic: {}", topic);
            for (id, deq) in map {
                println!("\t{} : {:?}", id, deq);
            }
        }
    }
}

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

fn send_message(socket: &zmq::Socket, id: &String, message: &str) {
    socket.send(id.as_bytes(), zmq::SNDMORE).unwrap();
    socket.send("", zmq::SNDMORE).unwrap();
    socket.send(message, 0).unwrap();
}

fn parse_request(request_id: &String, request: String, socket: &zmq::Socket, topics: &mut HashMap<String, HashMap<String, VecDeque<String>>>) {
    let split = request.splitn(3, " ");

    let vec: Vec<_> = split.collect();

    match vec[0] {
        "SUB" => {
            // Check if topic already exists
            if topics.contains_key(vec[1]) {
                let topic_map = topics.get_mut(vec[1]).unwrap();

                // Check if topic does not have the susbcriber
                if !topic_map.contains_key(request_id) {
                    topic_map.insert(String::from(request_id), VecDeque::new());
                }
            } else {
                let mut topic_map: HashMap<String, VecDeque<String>> = HashMap::new();
                topic_map.insert(String::from(request_id), VecDeque::new());

                topics.insert(vec[1].to_string(), topic_map);
            }

            send_message(&socket, &request_id, "OK");
        },

        "UNSUB" => {
            // Check if topic exists
            if topics.contains_key(vec[1]) {
                let topic_map = topics.get_mut(vec[1]).unwrap();

                topic_map.remove(request_id);

                send_message(&socket, &request_id, "OK");
            } else {
                send_message(&socket, &request_id, "NOK");
            }
        },

        "GET" => {
            // Check if topic already exists
            if topics.contains_key(vec[1]) {
                let topic_map = topics.get_mut(vec[1]).unwrap();

                // Check if topic does not have the susbcriber
                if topic_map.contains_key(request_id) {
                    let value = topic_map.get_mut(request_id).unwrap().pop_front();
                    if value == None {
                        send_message(&socket, &request_id, "OK");
                    } else {
                        send_message(&socket, &request_id, format!("OK {}", value.unwrap()).as_str());
                    }
                } else {
                    send_message(&socket, &request_id, "NOK");
                }
            } else {                
                send_message(&socket, &request_id, "NOK");
            }
        },

        "PUT" => {
            if topics.contains_key(vec[1]) {

                let topic_map = topics.get_mut(vec[1]).unwrap();

                for (_, queue) in topic_map {
                    queue.push_back(String::from(vec[2]));
                }

            } else {
                let topic_map: HashMap<String, VecDeque<String>> = HashMap::new();
                topics.insert(vec[1].to_string(), topic_map);
            }

            send_message(&socket, &request_id, "OK");
        },

        _ => {
            send_message(&socket, &request_id, "NOK");
        },
    }

}
