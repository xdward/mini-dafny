use std::env;
use std::fs;
use std::process;
use verifier;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: {} <file.txt>", &args[0]);
        process::exit(1);
    }

    let file_path = &args[1];
    let text = fs::read_to_string(file_path).expect("failed to read file");

    let result = verifier::verify(&text);
    println!("{}", result.message);
}
