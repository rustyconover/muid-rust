# Muid-Rust

Implement searching for [Memorable Unique Identifiers](https://github.com/microprediction/muid) (Muids) using Rust.

## Usage

Set the DIFFICULTY environment variable to the level of difficulty for the muids you'd like to
find.

Build the code using `cargo build --release` so you get an optimized build.

```
Muid Search Tool
Rusty Conover <rusty@conover.me>
Searches for Memorable Unique Identifiers of a specified difficulty.

USAGE:
    muid [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -m, --mode <mode>              Sets the mode used for searching [default: range]  [possible values: rng, range]
        --range-start <VALUE>      Sets the starting step number for range based searching [default: 0]
        --rng-max-tries <VALUE>    Sets the maximum number of iterations for rng base, 0 implies no limit [default: 0]
```

```sh
# For level 10, valid levels even numbers between 6-14
$ RUSTFLAGS="-C target-cpu=native" DIFFICULTY=10 cargo build --release
...
...

$ ./target/release/muid-rust
Searching with difficulty=6
Using 16 cpus to search for muids...

Pretty Name dog-fly Key: 0e09bdaa2a53d31aa35242ae50152f25 Hash: d09f14004105432722cda46ad08f4a4f
Pretty Name thy-dog Key: de52b52b10d6244f739688f61fca2b0d Hash: 764d09826b1292c2f79e83c2da56fb59
Pretty Name cal-cod Key: 0c86f987ec9a978039edfef34e5892f5 Hash: ca1c0d27ebbaf2111e03ae97b0fbf071
Pretty Name ebb-fox Key: 9b701fa73e12156302b3748f0fd77aaf Hash: ebbf0811b004ef1c6c334696e11663ce
Pretty Name hah-bat Key: 3e0a9c2b22000198d0495025ee3e3b07 Hash: 6a6ba7cfd42a44ee91591bb77e07f683
```

Or more challenging at difficulty 10

```
Pretty Name lactose-bat Key: 6834037f64447ce3ee783fada788c952 Hash: 1ac705eba79ecd4f0689cc07869d242b
Pretty Name sachem-toad Key: 3849a7a599274bd587717d5d2680f6d2 Hash: 5ac6e370addbd9086524ddc1789970fa
Pretty Name dambose-fox Key: d2d88c544ed6811ba431c311c7f37960 Hash: da3b05ef084ae4614ab9828bde0957ed
Pretty Name hatable-cod Key: 0ac7f3e7d0115ae516483ccd583adb24 Hash: 6a7ab1ec0d47ff7b19defe707e6016fc
```

Useful muids are of difficulty 12 and above.

For each increase in difficulty it takes about 16 times more CPU time to find one muid.

## Things of which you should be aware:

This program never terminates after finding a fixed number of muids, it just keeps searching.

You can stop searching if you're using the rng search method and specify the appropriate
parameter.

This progrmam will use all of the CPUs your computer has at 100%.
