# Muid-Rust

A implementation of mining [Memorable Unique Identifiers](https://github.com/microprediction/muid) (Muids) in Rust.

Created as a fun way to keep my Rust skills sharp.

Author: Rusty Conover <rusty@conover.me>

## Usage

Edit `src/main.rs` and set the DIFFICULTY constant to the difficulty of the muids you'd like to generate.

Build the code using `cargo build --release` so you get an optimized build.

```
$ ./target/release/muid-rust
Full corpus size=422013 Filtered corpus size=7019
Searching with difficulty=6
Using 16 cpus to search for muids...

Pretty Name dog-fly Key: 0e09bdaa2a53d31aa35242ae50152f25 Hash: d09f14004105432722cda46ad08f4a4f
Pretty Name thy-dog Key: de52b52b10d6244f739688f61fca2b0d Hash: 764d09826b1292c2f79e83c2da56fb59
Pretty Name cal-cod Key: 0c86f987ec9a978039edfef34e5892f5 Hash: ca1c0d27ebbaf2111e03ae97b0fbf071
Pretty Name ebb-fox Key: 9b701fa73e12156302b3748f0fd77aaf Hash: ebbf0811b004ef1c6c334696e11663ce
Pretty Name hah-bat Key: 3e0a9c2b22000198d0495025ee3e3b07 Hash: 6a6ba7cfd42a44ee91591bb77e07f683
```

Typically useful muids are of difficulty 12 and above.

For each increase in difficulty it takes about 16 times more CPU time to find one muid.

## Things of which you should be aware:

This program never terminates after finding a fixed number of muids, it just keeps searching.

This program will use all of the CPUs your computer has at 100%.
