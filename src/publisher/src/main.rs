use provider;

fn main() {
    let socket = provider::connect_publisher();

    provider::put(socket, "classes", "Helloooo");
}