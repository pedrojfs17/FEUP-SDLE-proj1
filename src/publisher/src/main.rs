use provider;
use std::thread;
use std::time;

fn main() {
    let socket = provider::connect_publisher();

    let mut i = 1;

    loop {
        provider::put(&socket, "classes", format!("Message {}", i).as_str());

        thread::sleep(time::Duration::from_secs(2));

        provider::put(&socket, "classes too", format!("Message {}", i).as_str());

        thread::sleep(time::Duration::from_secs(2));

        i = i + 1;

        if i == 11 {
            break;
        }
    }
}