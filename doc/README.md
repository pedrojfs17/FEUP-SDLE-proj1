# Project 1 - Reliable Pub/Sub Service

## Instalation

This project was built using [Rust](https://www.rust-lang.org/). Please use the following commands to install Rust in Linux:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

For the instalation: 

```
apt-get install libzmq3-dev
```

## Compilation

In order to compile the source code, please run the following command from the `src` folder:
```
cargo build
```

## Execution

### Server

In order to start the server please run the following command from the `src` folder:
```
cargo run -p server -- [-r|--reset-state]
```

Where:
- `<r>` or `reset-state` resets the server state .

### Client (Subscriber/Publisher)

In order to start a client please run the following command from the `src` folder:
```
cargo run -p client -- <id> [<mode>]
```
Where:
- `<id>` is the client's ID.
- `<mode>` is the type of client, subscriber or publisher. If this parameter is omitted, the client is
interactive and the user can change its behaviour.