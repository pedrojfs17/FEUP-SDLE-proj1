# Project 1 - Reliable Pub/Sub Service

## Compilation

In order to compile the source code, please run the following command from the `src` folder:
```
cargo build
```

## Execution

### Server

In order to start the server please run the following command from the `src` folder:
```
cargo run -p server
```

### Subscriber

In order to start a subscriber please run the following command from the `src` folder:
```
cargo run -p subscriber -- <id>
```
Where:
- `<id>` is the subscriber's ID

### Publisher

In order to start a publisher please run the following command from the `src` folder:
```
cargo run -p publisher
```