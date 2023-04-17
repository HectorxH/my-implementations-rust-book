use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
    thread,
    time::Duration,
};

use hello::ThreadPool;

#[derive(Debug, PartialEq, Eq)]
enum HTTPMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

impl FromStr for HTTPMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<HTTPMethod, Self::Err> {
        match s {
            "GET" => Ok(HTTPMethod::GET),
            "POST" => Ok(HTTPMethod::POST),
            "PUT" => Ok(HTTPMethod::PUT),
            "DELETE" => Ok(HTTPMethod::DELETE),
            _ => Err(()),
        }
    }
}

fn main() {
    let listener =
        TcpListener::bind("127.0.0.1:7878").expect("Should be able to bind to ip 127.0.0.1:7878");
    let mut thread_pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => thread_pool.execute(|| handle_connection(stream)),
            Err(err) => eprintln!("Connection failed: {err:#?}"),
        };
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request = match buf_reader.lines().next() {
        Some(Ok(http_request)) => http_request,
        Some(Err(err)) => {
            eprintln!("Found error while reading from buffer. {err:#?}");
            return;
        }
        None => {
            eprintln!("Invalid HTTP request line.");
            return;
        }
    };

    let http_request: Vec<&str> = http_request.split(" ").collect();

    let [method, route, _version] = http_request[..] else {
        eprintln!("Invalid HTTP request line.");
        return;
    };

    println!("{method} {route}");

    let (status_line, filename) = match (method, route) {
        ("GET", "/") => ("HTTP/1.1 200 OK", "src/hello.html"),
        ("GET", "/sleep") => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "src/hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "src/404.html"),
    };

    // The file filename should exist as it's defined in the code above.
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    if let Err(err) = stream.write_all(response.as_bytes()) {
        eprintln!("Couldn't send response.\nError: {err:#?}");
    };
}
