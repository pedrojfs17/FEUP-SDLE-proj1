use std::env;
use provider;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Error: Subscribers must have an ID!\n\tUsage: ./subscriber <id>");
        return;
    }

    let socket = provider::connect_subscriber(args[1].as_bytes());

    provider::subscribe(socket, "classes");
}
