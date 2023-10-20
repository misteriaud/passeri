Passeri is a [MIDI](https://en.wikipedia.org/wiki/MIDI) **Sender**/**Receiver** bridge over Network (*TCP*, *Bluetooth*, and other to come).

It is build on a parallel blocking architecture composed of 3 threads:
- `main_thread` where the bridge reside
- `midi_thread` which is in charge of communication with local midi ports
- `network_thread` which forward or receive MIDI messages on given network

**Passeri** is shipped with a couple of helper functions to make it easier to use.

# Roadmap
- [X] MIDI thread implementation
	- [x] MIDI sender
	- [x] MIDI receiver
- [X] Bridge api development ([passeri-api](passeri-api))
	- [x] Trait description
	- [x] Sender and Receiver implementation
	- [x] Documentation
- [ ] TCP implementation ([passeri-tcp](passeri-tcp))
	- [X] PoC
	- [ ] Documentation
	- [ ] Testing
	- [ ] Benchmark
- [ ] Bluetooth implementation ([passeri-bluetooth](passeri-bluetooth)) following [BLE MIDI](https://hangar42.nl/wp-content/uploads/2017/10/BLE-MIDI-spec.pdf)
	- [ ] PoC
	- [ ] Documentation
	- [ ] Testing
	- [ ] Benchmark
- [ ] GUI using Tauri

# Examples
Some examples are available in the `examples/` directory.

## A Simple TCP Sender and Receiver
A very simple set of Sender and Receiver using TCP Network can be run as following:

### Sender
First let's run the Sender:
```sh
cargo run --example sender 127.0.0.1:8080
```

### Receiver
you can then connect to it using the following command
```sh
cargo run --example receiver -- 127.0.0.1:8080
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
