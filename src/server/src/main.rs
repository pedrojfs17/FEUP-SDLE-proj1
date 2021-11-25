use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::fs::File;
use std::str;
use std::time;
use std::env;
use serde_json;
use scheduled_thread_pool;
#[macro_use] extern crate lazy_static;

// Testing
// use std::thread;

lazy_static!(
    static ref TOPICS: Mutex<HashMap<String, HashMap<String, VecDeque<String>>>> = Mutex::new(HashMap::new());
    static ref PENDING_REQUESTS: Mutex<HashMap<String, HashSet<String>>> = Mutex::new(HashMap::new());
);

fn main() {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::ROUTER).unwrap();
    socket.set_router_mandatory(true).unwrap();

    socket
        .bind("tcp://*:5559")
        .expect("failed binding socket");

    let args: Vec<String> = env::args().collect();

    println!("Server Starting!");

    if !(args.len() == 2 && (args[1] == "-r" || args[1] == "--reset-status")) {
        recover_state();
    }

    println!("Server Online!");

    let scheduled_thread_pool = scheduled_thread_pool::ScheduledThreadPool::new(1);

    scheduled_thread_pool.execute_at_fixed_rate(time::Duration::from_secs(5), time::Duration::from_secs(5), || {
        let topics_file: File = File::create("topics.json").unwrap();
        serde_json::to_writer(&topics_file, &*TOPICS).unwrap();

        let pendings_file: File = File::create("pending.json").unwrap();
        serde_json::to_writer(&pendings_file, &*PENDING_REQUESTS).unwrap();
    });

    loop {
        let (request_id, request) = read_message(&socket);

        parse_request(&request_id, request, &socket);

        print_topics();
        print_pending_requests();
    }
}

fn print_topics() {
    println!("---------- TOPICS ----------");
    for (topic, map) in TOPICS.lock().unwrap().iter() {
        println!("-> {}", topic);
        for (id, deq) in map {
            println!("\t{} : {:?}", id, deq);
        }
    }
    println!("----------------------------\n");
}

fn print_pending_requests() {
    println!("----- PENDING REQUESTS -----");
    for (topic, set) in PENDING_REQUESTS.lock().unwrap().iter() {
        println!("-> {} : {:?}", topic, set);
    }
    println!("----------------------------\n");
}

fn recover_state() {
    println!("Recovering State...");

    let file = File::open("topics.json");

    if file.is_ok() {
        *TOPICS.lock().unwrap() = serde_json::from_reader(file.unwrap()).unwrap();
        print_topics();
    } else {
        println!("Topics state not found! Creating a new one...")
    }

    let file = File::open("pending.json");

    if file.is_ok() {
        *PENDING_REQUESTS.lock().unwrap() = serde_json::from_reader(file.unwrap()).unwrap();
        print_pending_requests();
    } else {
        println!("Pending requests state not found! Creating a new one...")
    }
}

fn read_message(socket: &zmq::Socket) -> (String, String) {
    let result = socket.recv_multipart(0);

    match result {
        Ok(message) => {
            let id = match str::from_utf8(&message[0]) {
                Ok(v) => String::from(v),
                Err(_e) => "".to_string(),
            };

            let request = match str::from_utf8(&message[2]) {
                Ok(v) => String::from(v),
                Err(_e) => "".to_string(),
            };

            println!("Received request from: {} -> {}", id, request);
            return (id, request);
            
        },
        Err(e) => {
            println!("Read Error: {:?}", e);
            return ("".to_string(), "".to_string());
        },
    }
}

fn send_message(socket: &zmq::Socket, id: &String, message: &str) -> i32 {
    let result = socket.send_multipart([id.as_bytes(), "".as_bytes(), message.as_bytes()], 0);

    match result {
        Ok(_v) => {},
        Err(e) => {
            println!("Send Error: {:?}", e);
            return -1;
        },
    }

    return 0;
}

fn parse_request(request_id: &String, request: String, socket: &zmq::Socket) {
    let mut topics = TOPICS.lock().unwrap();

    let split: Vec<_> = request.splitn(2, " ").collect();

    let action = split[0];

    let start_bytes; let mut end_bytes = 0; let mut topic = "";

    if split.len() >= 2 {
        start_bytes = split[1].find("[").unwrap_or(0) + 1;
        end_bytes = split[1].find("]").unwrap_or(split[1].len());
        topic = &split[1][start_bytes..end_bytes];
    }

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

                if topic_map.keys().len() == 0 {
                    topics.remove(topic);
                }
            }
            
            send_message(&socket, &request_id, "OK");
        },

        "GET" => {
            // Check if topic already exists
            if topics.contains_key(topic) {
                let topic_map = topics.get_mut(topic).unwrap();

                // Check if topic does not have the susbcriber
                if topic_map.contains_key(request_id) {
                    let front = topic_map.get_mut(request_id).unwrap().pop_front();
                    if front == None {
                        add_pending_request(String::from(topic), String::from(request_id));
                    } else {
                        let value = front.unwrap();
                        if send_message(&socket, &request_id, format!("OK {}", value).as_str()) == -1 {
                            topic_map.get_mut(request_id).unwrap().push_front(value);
                        }
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

                drop(topics);

                check_pending_requests(String::from(topic), socket);
            } 

            send_message(&socket, &request_id, "OK");
        },

        "ONLINE" => {
            let mut pending_requests = PENDING_REQUESTS.lock().unwrap();
            let mut topics_to_remove: Vec<String> = Vec::new();

            for (topic, set) in pending_requests.iter() {
                if set.contains(request_id) {
                    println!("User {} was waiting for topic [{}] but has crashed.", request_id, topic);
                    topics_to_remove.push(String::from(topic));
                }
            }

            for topic in topics_to_remove {
                let topics_set = pending_requests.get_mut(&topic).unwrap();
                topics_set.remove(request_id);
                if topics_set.len() == 0 {
                    pending_requests.remove(&topic);
                }
            }

            send_message(&socket, &request_id, "OK");
        },

        _ => {
            send_message(&socket, &request_id, "NOK");
        },
    }

}

fn add_pending_request(topic: String, id: String) {
    let mut pending_requests = PENDING_REQUESTS.lock().unwrap();

    if pending_requests.contains_key(&topic) {
        pending_requests.get_mut(&topic).unwrap().insert(id);
    } else {
        let mut set: HashSet<String> = HashSet::new();
        set.insert(id);
        pending_requests.insert(topic, set);
    }
}

fn check_pending_requests(topic: String, socket: &zmq::Socket) {
    let mut topics = TOPICS.lock().unwrap();
    let mut pending_requests = PENDING_REQUESTS.lock().unwrap();

    if pending_requests.contains_key(&topic) {
        let requests = pending_requests.get_mut(&topic).unwrap();
        let topic_map = topics.get_mut(&topic).unwrap();

        for id in requests.iter() {
            let value = topic_map.get_mut(id).unwrap().pop_front().unwrap();

            if send_message(&socket, &id, format!("OK {}", value).as_str()) == -1 {
                topic_map.get_mut(id).unwrap().push_front(value);
            }
        }

        pending_requests.remove(&topic);
    }
}

