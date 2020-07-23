#![feature(proc_macro_hygiene)]

use hex::encode;
use random_fast_rng::{local_rng, Random};
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

fn mine() {
    let mut gen = local_rng();
    let mut pool: [u8; 16 * POOL_SIZE] = [0; 16 * POOL_SIZE];
    let mut encoded_pool: [u8; 2 * 16 * POOL_SIZE] = [0; 2 * 16 * POOL_SIZE];

    loop {
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
    }
}

fn main() {
    if DIFFICULTY % 2 == 1 {
        println!("Difficulty must be even");
        process::exit(0x0100);
    }

    println!("Searching with difficulty={}", DIFFICULTY);

    let cpus = num_cpus::get();
    println!("Using {} cpus to search for muids...", cpus);
    println!("");

    let threads: Vec<_> = (0..cpus).map(|_i| thread::spawn(move || mine())).collect();

    // The threads never exist, but leave this here for now.
    for t in threads {
        t.join().expect("Thread panicked");
    }
}
