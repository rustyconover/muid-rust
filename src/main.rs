#![feature(proc_macro_hygiene)]

use clap::{App, Arg};
use hex::encode;
use rand::prelude::*;
use rayon::prelude::*;
use ring::digest;
use std::process;
use std::str;
use std::thread;

// This is the difficulty.
//
// it only works for even numbers due to the mine() function
// not dealing with filtering high and low bits of a byte.
//
// Since 12 seems to be the minimum difficulty for now to write
// new streams this seems acceptable.
include!(concat!(env!("OUT_DIR"), "/difficulty.rs"));

fn to_readable_hex(word: &str) -> String {
    word.chars()
        .map(|x| match x {
            '0' => 'o',
            '1' => 'l',
            '2' => 'z',
            '3' => 'm',
            '4' => 'y',
            '5' => 's',
            '6' => 'h',
            '7' => 't',
            '8' => 'x',
            '9' => 'g',
            _ => x,
        })
        .collect()
}

fn pretty_animal_name(code: &str, l1: u8, l2: u8) -> String {
    let first = to_readable_hex(&code[0..l1 as usize]);
    let second = to_readable_hex(&code[(l1 as usize)..(l1 as usize) + (l2 as usize)]);
    return format!("{}-{}", first, second);
}

fn report_finding(key: &str, code: &str, kc: &(u8, u8)) {
    let pretty = pretty_animal_name(code, kc.0, kc.1);
    let full_code = bhash(key.as_bytes());
    println!("Pretty Name {} Key: {} Hash: {}", pretty, key, full_code)
}

// Take a string and return a 32 byte hex respresentation of the sha256 of that string.
fn bhash(key: &[u8]) -> String {
    // create a Sha256 object
    let result = digest::digest(&digest::SHA256, key);
    // Take the first 16 bytes and encode them as hex.
    return encode(&result.as_ref()[0..16]);
}

const POOL_SIZE: usize = 256 * 128;

fn generate_iter(len: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..len).step_by(2).zip((0..len).skip(1).step_by(2))
}

fn byte2hex(byte: u8, table: &[u8; 16]) -> (u8, u8) {
    let high = table[((byte & 0xf0) >> 4) as usize];
    let low = table[(byte & 0x0f) as usize];

    (high, low)
}
const HEX_CHARS_LOWER: &[u8; 16] = b"0123456789abcdef";

fn mine_using_ranges(start_step: u128) {
    let step_size = 1024 * 1024;
    let step_count = u128::MAX / step_size;
    let iterator = start_step..step_count;

    println!(
        "Using range search, step size: {}, total steps for range: {}...",
        step_size, step_count
    );

    // The iterator does not process steps in a guaranteed order
    // but using Rayon this way proceeds that the same value
    // is never tested twice.
    //
    // This may make suspending and resuming the searches more
    // productive.
    iterator.into_par_iter().for_each(|start| {
        let mut dest: [u8; 32] = [0; 32];
        for val in start * step_size..(start + 1) * step_size {
            hex::encode_to_slice(val.to_be_bytes(), &mut dest).unwrap();

            // Now sha256 those bytes and get a string
            let result = digest::digest(&digest::SHA256, &dest);

            // search for a prefix of the sha256 in the corpus
            let short_hash = &result.as_ref()[0..DIFFICULTY / 2];
            let short_result = include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

            if short_result.is_some() {
                report_finding(
                    &str::from_utf8(&dest).unwrap(),
                    &bhash(&dest)[0..DIFFICULTY],
                    &short_result.unwrap(),
                );
            }
        }
    });
}

fn mine_using_rng(max_tries: u128) {
    let mut gen = rand::thread_rng();
    let mut pool: [u8; 16 * POOL_SIZE] = [0; 16 * POOL_SIZE];
    let mut encoded_pool: [u8; 2 * 16 * POOL_SIZE] = [0; 2 * 16 * POOL_SIZE];
    let mut counter = if max_tries == 0 {
        std::u128::MAX
    } else {
        max_tries
    };
    while counter > 0 {
        // Fill up the pool of random bytes.
        gen.fill_bytes(&mut pool);

        for (byte, (i, j)) in pool
            .as_ref()
            .iter()
            .zip(generate_iter(pool.as_ref().len() * 2))
        {
            let (high, low) = byte2hex(*byte, HEX_CHARS_LOWER);
            encoded_pool[i] = high;
            encoded_pool[j] = low;
        }

        for i in 0..POOL_SIZE - 32 {
            // Chunk over the random bytes
            let random_32_byte_hex = &encoded_pool[i..i + 32];
            // Now sha256 those bytes and get a string
            let result = digest::digest(&digest::SHA256, random_32_byte_hex);

            // search for a prefix of the sha256 in the corpus
            let short_hash = &result.as_ref()[0..DIFFICULTY / 2];
            let short_result = include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

            if short_result.is_some() {
                report_finding(
                    &str::from_utf8(random_32_byte_hex).unwrap(),
                    &bhash(random_32_byte_hex)[0..DIFFICULTY],
                    &short_result.unwrap(),
                );
            }
        }
        counter = counter - 1;
    }
}

fn main() {
    if DIFFICULTY % 2 == 1 {
        println!("Difficulty must be even");
        process::exit(0x0100);
    }

    let matches = App::new("Muid Search Tool")
        .author("Rusty Conover <rusty@conover.me>")
        .about("Searches for Memorable Unique Identifiers of a specified difficulty.")
        .arg(
            Arg::with_name("mode")
                .short("m")
                .long("mode")
                .default_value("range")
                .possible_value("rng")
                .possible_value("range")
                .help("Sets the mode used for searching")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("range-start")
                .long("range-start")
                .default_value("0")
                .value_name("VALUE")
                .help("Sets the starting step number for range based searching")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("rng-max-tries")
                .long("rng-max-tries")
                .default_value("0")
                .value_name("VALUE")
                .help("Sets the maximum number of iterations for rng base, 0 implies no limit")
                .takes_value(true),
        )
        .get_matches();

    let mode = matches.value_of("mode").unwrap_or("range");

    let range_start_str = matches
        .value_of("range-start")
        .unwrap_or("0")
        .parse::<u128>()
        .unwrap();

    let rng_max_tries = matches
        .value_of("rng-max-tries")
        .unwrap_or("0")
        .parse::<u128>()
        .unwrap();

    println!("Searching with difficulty={}", DIFFICULTY);

    let cpus = num_cpus::get();
    println!("Using {} cpus to search for muids...", cpus);
    println!("");

    if mode == "range" {
        // Range based mining with Rayon.
        mine_using_ranges(range_start_str);
    } else {
        // Random number based mining.
        let threads: Vec<_> = (0..cpus)
            .map(|_i| thread::spawn(move || mine_using_rng(rng_max_tries)))
            .collect();

        // The threads never exist, but leave this here for now.
        for t in threads {
            t.join().expect("Thread panicked");
        }
    }
}
