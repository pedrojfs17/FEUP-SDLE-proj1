use std::thread::sleep;
use std::time::Duration;

fn main() {
    let context = zmq::Context::new();
    let publisher = context.socket(zmq::PUB).unwrap();

    publisher
        .connect("tcp://localhost:5559")
        .expect("failed connecting publisher");

    loop {
        sleep(Duration::from_millis(1000));
        publisher
            .send("sdle", zmq::SNDMORE)
            .unwrap();
        publisher
            .send("I love this curricular unit!", 0)
            .unwrap();

        sleep(Duration::from_millis(1000));
        publisher
            .send("testing", zmq::SNDMORE)
            .unwrap();
        publisher
            .send("I am only testing!", 0)
            .unwrap();
    }
}