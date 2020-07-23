#![feature(proc_macro_hygiene)]

use fnv::FnvHashMap;
use hex::{decode, encode};
use random_fast_rng::{local_rng, Random};
use serde_json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::process;
use std::sync::{Arc, RwLock};
use std::thread;

// This is the difficulty.
//
// it only works for even numbers due to the mine() function
// not dealing with filtering high and low bits of a byte.
//
// Since 12 seems to be the minimum difficulty for now to write
// new streams this seems acceptable.
const DIFFICULTY: usize = 12;

// Invert the readable hex
fn from_readable_hex(word: &str) -> String {
    word.chars()
        .map(|x| match x {
            'o' => '0',
            'l' => '1',
            'z' => '2',
            'm' => '3',
            'y' => '4',
            's' => '5',
            'h' => '6',
            't' => '7',
            'x' => '8',
            'g' => '9',
            x => x,
        })
        .collect()
}

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
    let full_code = bhash(&key);
    println!("Pretty Name {} Key: {} Hash: {}", pretty, key, full_code)
}

// Take a string and return a 32 byte hex respresentation of the sha256 of that string.
fn bhash(key: impl AsRef<[u8]>) -> String {
    // create a Sha256 object
    let mut hasher = Sha256::new();
    // write input message
    hasher.update(key);
    // read hash digest and consume hasher
    let result = hasher.finalize();
    // Take the first 16 bytes and encode them as hex.
    return encode(&result.as_slice()[0..16]);
}

// Hash the value but don't hex encode it, place it in
// a destination rather than allocating a string.
fn bhash2(key: impl AsRef<[u8]>, dest: &mut [u8; 16]) {
    let mut hasher = Sha256::new();
    hasher.update(key);
    let result = hasher.finalize();
    dest.copy_from_slice(&result.as_slice()[0..16]);
}

fn mine(corpus: &Arc<RwLock<FnvHashMap<[u8; DIFFICULTY / 2], (u8, u8)>>>) {
    let map = corpus.read().expect("RwLock poisoned");
    let mut target: [u8; 16] = [0; 16];
    let mut gen = local_rng();
    loop {
        // Generate 16 random bytes, then convert them to hex
        //
        // Is searching 16 random bytes the most productive way to find
        // these values? With more CPUs the search could be more coordinated
        // so that values could be saved and resumed, but alas I'll go
        // with this for now.
        //
        let random_32_byte_hex = encode(gen.gen::<[u8; 16]>());
        // Now sha256 those bytes and get a string
        bhash2(&random_32_byte_hex, &mut target);
        // search for a prefix of the sha256 in the corpus
        let short_result = map.get(&target[0..DIFFICULTY / 2]);
        if short_result.is_some() {
            // Found a hit.
            report_finding(
                &random_32_byte_hex,
                &bhash(&random_32_byte_hex)[0..DIFFICULTY],
                short_result.unwrap(),
            );
        }
    }
}

fn main() {
    // This is pretty lazy to load a file and use serde to just load it all
    // into memory, but its okay.
    let mut file = File::open("animals.json").unwrap();
    let mut animal_content = String::new();
    file.read_to_string(&mut animal_content).unwrap();
    let full_corpus: HashMap<String, (u8, u8)> = serde_json::from_str(&animal_content).unwrap();

    if DIFFICULTY % 2 == 1 {
        println!("Difficulty must be even");
        process::exit(0x0100);
    }
    // The only keys that should be in the hash should be the ones with the
    // known difficulty.
    let mut search_corpus: FnvHashMap<[u8; DIFFICULTY / 2], (u8, u8)> =
        FnvHashMap::with_capacity_and_hasher(8000, Default::default());

    full_corpus
        .iter()
        .filter(|v| v.0.len() == DIFFICULTY)
        .for_each(|v| {
            let full_hex = decode(from_readable_hex(v.0));
            // sometimes the corpus contains values that aren't
            // valid readable hex values, so ignore those.
            if full_hex.is_ok() {
                search_corpus.insert(full_hex.unwrap().as_slice().try_into().unwrap(), *v.1);
            }
        });

    println!(
        "Full corpus size={} Filtered corpus size={}",
        full_corpus.len(),
        search_corpus.len()
    );
    println!("Searching with difficulty={}", DIFFICULTY);

    // Make the corpus thread friendly.
    let running_corpus = Arc::new(RwLock::new(search_corpus));

    let cpus = num_cpus::get();
    println!("Using {} cpus to search for muids...", cpus);
    println!("");

    let threads: Vec<_> = (0..cpus)
        .map(|_i| {
            let data = Arc::clone(&running_corpus);
            thread::spawn(move || mine(&data))
        })
        .collect();

    // The threads never exist, but leave this here for now.
    for t in threads {
        t.join().expect("Thread panicked");
    }
}
