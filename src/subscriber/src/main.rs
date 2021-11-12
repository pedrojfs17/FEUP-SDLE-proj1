fn main() {
    let context = zmq::Context::new();
    let subscriber = context.socket(zmq::SUB).unwrap();

    subscriber
        .connect("tcp://localhost:5560")
        .expect("failed to connect subscriber");

    let subscriptions = vec!["testing".as_bytes(), "sdle".as_bytes()];

    subscriber.set_subscribe(subscriptions[0]).unwrap();
    subscriber.set_subscribe(subscriptions[1]).unwrap();

    loop {
        let topic = subscriber.recv_msg(0).unwrap();
        let data = subscriber.recv_msg(0).unwrap();
        assert!(&subscriptions.contains(&&topic[..]));
        println!("[{}] {}", topic.as_str().unwrap(), data.as_str().unwrap());
    }
}
