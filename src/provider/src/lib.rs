pub fn connect_subscriber(id: &[u8]) -> zmq::Socket {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::REQ).unwrap();

    socket.set_identity(id).unwrap();

    socket
        .connect("tcp://localhost:5559")
        .expect("failed to connect subscriber");
    
    return socket;
}

pub fn connect_publisher() -> zmq::Socket {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::REQ).unwrap();
    
    socket.set_identity("PUB".as_bytes()).unwrap();

    socket
        .connect("tcp://localhost:5559")
        .expect("failed to connect publisher");
    
    return socket;
}

pub fn subscribe(socket: &zmq::Socket, topic: &str) {
    println!("Subscribing topic: {}", topic);

    let message = format!("SUB {}", topic);
    message_to_server(socket, &message);
}

pub fn unsubscribe(socket: &zmq::Socket, topic: &str) {
    println!("Unsubscribing topic: {}", topic);

    let message = format!("UNSUB {}", topic);
    message_to_server(socket, &message);
}

pub fn get(socket: &zmq::Socket, topic: &str) {
    println!("Get topic: {}", topic);

    let message = format!("GET {}", topic);
    message_to_server(socket, &message);
}

pub fn put(socket: &zmq::Socket, topic: &str, value: &str) {
    println!("Put in topic: {} value: {}", topic, value);

    let message = format!("PUT {} {}", topic, value);
    message_to_server(socket, &message);
}

fn message_to_server(socket: &zmq::Socket, message: &str) {
    socket.send(&message, 0).unwrap();

    let response = socket.recv_msg(0).unwrap();
    println!("Received reply: {}", response.as_str().unwrap());
}
