use hex::decode;
use serde_json;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufWriter, Write};
use std::path::Path;

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

fn main() {
    let difficulty: usize = env::var("DIFFICULTY").unwrap().parse::<usize>().unwrap();
    println!("cargo:rerun-if-env-changed=DIFFICULTY");
    println!("cargo:rerun-if-changed=animals.json");
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("difficulty.rs");
    let mut file3 = BufWriter::new(File::create(&path).unwrap());
    writeln!(&mut file3, "const DIFFICULTY: usize = {};", difficulty).unwrap();

    let mut file2 = File::open("animals.json").unwrap();
    let mut animal_content = String::new();
    file2.read_to_string(&mut animal_content).unwrap();
    let full_corpus: HashMap<String, (u8, u8)> = serde_json::from_str(&animal_content).unwrap();

    let mut v: Vec<_> = full_corpus
        .into_iter()
        .filter(|v| v.0.len() == difficulty && decode(from_readable_hex(&v.0)).is_ok())
        .collect();
    v.sort_by(|x, y| x.0.cmp(&y.0));

    write!(&mut file, "match (").unwrap();
    for i in 0..difficulty / 2 {
        if i != 0 {
            write!(&mut file, ",").unwrap();
        }
        write!(&mut file, "short_hash[{}]", i).unwrap();
    }
    writeln!(&mut file, ") {{").unwrap();

    let mut items = v.iter().peekable();

    while let Some((k, v)) = items.next() {
        writeln!(&mut file, "// {}", k).unwrap();

        let full_hex = decode(from_readable_hex(k));
        // sometimes the corpus contains values that aren't
        // valid readable hex values, so ignore those.

        // So the next thing we should do is collapse the match arms.

        if full_hex.is_ok() {
            let h = full_hex.unwrap();
            write!(&mut file, "(").unwrap();
            for i in 0..difficulty / 2 {
                if i != 0 {
                    write!(&mut file, ",").unwrap();
                }

                write!(&mut file, "{}", h[i]).unwrap();
            }
            write!(&mut file, ")").unwrap();
            let next_val = items.peek();
            if next_val.is_some() && next_val.unwrap().1 == *v {
                write!(&mut file, " | ").unwrap();
            } else {
                writeln!(&mut file, " => Some(({}, {})),", v.0, v.1).unwrap();
            }
        }
    }

    write!(&mut file, "(").unwrap();
    for i in 0..difficulty / 2 {
        if i != 0 {
            write!(&mut file, ",").unwrap();
        }

        write!(&mut file, "_").unwrap();
    }
    writeln!(&mut file, ") => None",).unwrap();

    writeln!(&mut file, "}}\n",).unwrap();
}
