use provider;
use std::thread;
use std::time;

fn main() {
    let socket = provider::connect_publisher();

    let mut i = 1;

    loop {
        thread::sleep(time::Duration::from_secs(2));

        provider::put(&socket, "classes", format!("Message {}", i).as_str());

        i = i + 1;

        if i == 10 {
            break;
        }
    }
}