Passeri is a [MIDI](https://en.wikipedia.org/wiki/MIDI) **Sender**/**Receiver** over Network (*TCP*, *Bluetooth*, and other to come).

It is build on a parallel blocking architecture composed of 3 threads:
- `main_thread`
- `midi_thread`
- `network_thread`

This architecture have been chosen over concurrent one for its efficiency.

**Passeri** is shipped with a couple of helper functions to make it easier to use.

# Roadmap
- MIDI thread implementation
	- [x] MIDI sender
	- [x] MIDI receiver
- Network thread implementation
	- [x] modular binding using trait
	- [x] support of TCP
	- [ ] support of Bluetooth
- [ ] Perform benchmarks
- [ ] GUI using Tauri

# Examples
Some examples are available in the `examples/` directory.

## A Simple TCP Sender and Receiver
A very simple set of Sender and Receiver using TCP Network can be run as following:

### Sender
First let's run the Sender:
```sh
cargo run --example sender
```
It will print the address on which it is waiting for a receiver
e.g:
```
127.0.0.1:52452 is now connected
```

### Receiver
you can then connect to it using the following command
```sh
cargo run --example receiver -- 127.0.0.1:52452
```