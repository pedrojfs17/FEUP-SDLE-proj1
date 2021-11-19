use std::env;
use std::thread;
use std::time;
use provider;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Error: Subscribers must have an ID!\n\tUsage: ./subscriber <id>");
        return;
    }

    let socket = provider::connect_subscriber(args[1].as_bytes());

    provider::subscribe(&socket, "classes");

    let mut i = 0;

    loop {
        thread::sleep(time::Duration::from_secs(1));

        provider::get(&socket, "classes");

        i = i + 1;

        if i == 20 {
            break;
        }
    }

    provider::unsubscribe(&socket, "classes");
}
