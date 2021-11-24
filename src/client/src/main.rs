use std::io::{self, Write};
use std::env;
use std::thread;
use std::time;
use provider;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        println!("Error: Wrong number of arguments!\n\tUsage: ./client <id> [<mode>]");
        return;
    }

    let mode = if args.len() == 3 { 
        args[2].as_str() 
    } else { 
        "" 
    };

    match mode {
        // Subscriber
        "sub1" => sub1(args[1].as_str()),
        "sub2" => sub2(args[1].as_str()),

        // Publisher
        "pub1" => pub1(args[1].as_str()),

        // Intercative
        _ => interactive(args[1].as_str())
    }
}

fn interactive(id: &str) {
    let socket = provider::connect(id.as_bytes());

    loop {
        println!("----- Interactive Pub/Sub -----");
        println!("Send one of the following:");
        println!("- SUB <topic>");
        println!("- UNSUB <topic>");
        println!("- GET <topic>");
        println!("- PUT <topic>");
        println!("- QUIT");
        print!("Request: ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        line = String::from(line.trim());

        let v: Vec<&str> = line.splitn(2, " ").collect();

        if !v[0].chars().all(char::is_alphanumeric) {
            println!("Topics can not have non alphanumeric chars!");
            continue
        }

        match v[0].to_uppercase().as_str() {
            "SUB" => provider::subscribe(&socket, v[1]),
            "UNSUB" => provider::unsubscribe(&socket, v[1]),
            "GET" => provider::get(&socket, v[1]),
            "PUT" => {
                let mut msg = String::new();
                print!("Message: ");
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut msg).unwrap();
                provider::put(&socket, v[1], msg.as_str());
            },
            "QUIT" => {
                println!("Goodbye!");
                break
            },
            _ => {
                println!("Invalid Message!");
            }
        }

        println!();
    }
}

fn sub1(id: &str) {
    let socket = provider::connect(id.as_bytes());

    provider::subscribe(&socket, "classes");

    let mut i = 0;
    loop {
        provider::get(&socket, "classes");

        i = i + 1;

        if i == 10 {
            break;
        }
    }

    provider::unsubscribe(&socket, "classes");
}

fn pub1(id: &str) {
    let socket = provider::connect(id.as_bytes());

    let mut i = 0;
    loop {
        provider::put(&socket, "classes", format!("Message {}", i).as_str());

        thread::sleep(time::Duration::from_secs(2));

        i = i + 1;

        if i == 10 {
            break;
        }
    }
}

fn sub2(id: &str) {
    let socket = provider::connect(id.as_bytes());

    provider::subscribe(&socket, "classes");
    provider::subscribe(&socket, "classes too");

    let mut i = 0;

    loop {
        // thread::sleep(time::Duration::from_secs(1));

        if i % 2 == 0 {
            provider::get(&socket, "classes");
        } else {
            provider::get(&socket, "classes too");
        }

        i = i + 1;

        if i == 20 {
            break;
        }
    }

    provider::unsubscribe(&socket, "classes");
    provider::unsubscribe(&socket, "classes too");
}
