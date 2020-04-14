extern crate termion;
extern crate dqaunt_query_service_rust;

use std::fmt::Error;
use termion::input::TermRead;
use std::io::{Write, stdout, stdin};
use dqaunt_query_service_rust::parse_stdin;
use std::thread::sleep;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let stdin = stdin();
    let mut stdin = stdin.lock();

    loop {
        stdout.write_all(b"input: ").unwrap();
        stdout.flush().unwrap();

        let input = stdin.read_line();

        if let Ok(Some(input)) = input {
            let args: Vec<&str> = input.as_str().split_whitespace().collect();
            if args.len() == 0 {
                stdout.write_all("Please input something".as_bytes()).unwrap();
                stdout.write_all(b"\n").unwrap();
            } else {
                if let Err(e) = parse_stdin(args){
                    stdout.write_all(Error.to_string().as_bytes());
                    stdout.write_all(b"\n").unwrap();
                }
                sleep(Duration::from_millis(100));
            }
        } else {
            stdout.write_all(b"Error\n").unwrap();
        }
    }


}