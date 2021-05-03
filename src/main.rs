use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

#[derive(Debug, Eq, PartialEq)]
enum Request {
    Publish(String),
    Retrieve,
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let storage = Arc::new(Mutex::new(VecDeque::<String>::new()));

    for connection_attempt in listener.incoming() {
        match connection_attempt {
            Ok(stream) => {
                let mut thread_handle = Arc::clone(&storage);

                std::thread::spawn(move || {
                    handle_client(stream, &mut thread_handle);
                });
            }
            Err(e) => {
                eprintln!("Error connecting: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, storage: &Mutex<VecDeque<String>>) {
    println!("Client connected!");

    let line = read_line(&stream);
    let request = parse_request(line);

    match request {
        Request::Publish(msg) => {
            println!("publishing message");
            let mut guard = storage.lock().unwrap();
            guard.push_back(msg);
        }
        Request::Retrieve => {
            println!("retrieving message");
            let maybe_msg = storage.lock().unwrap().pop_front();
            match maybe_msg {
                Some(msg) => stream.write_all(msg.as_bytes()).unwrap(),
                None => stream.write_all(b"no message available").unwrap(),
            }
        }
    }
}

fn read_line(stream: &TcpStream) -> String {
    let mut buffered_reader = BufReader::new(stream);

    let mut buf = String::new();
    buffered_reader.read_line(&mut buf).unwrap();

    buf
}

fn parse_request(line: String) -> Request {
    let trimmed = line.trim_end();

    if trimmed == "" {
        Request::Retrieve
    } else {
        Request::Publish(String::from(trimmed))
    }
}
