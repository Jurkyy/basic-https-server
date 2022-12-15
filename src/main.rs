use basic_https_server::ThreadPool;

extern crate native_tls;
use native_tls::{Identity, TlsAcceptor, TlsStream};

use std::{
    fs,
    fs::File,
    io::{prelude::*, BufReader, Result, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
    time::Duration,
};

fn handle_connection(mut stream: TlsStream<TcpStream>) -> Result<()> {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "templates/hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "templates/hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "templates/404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    Ok(())
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8443")?;
    println!("Listening for connections on port 8443");

    let mut file = File::open("cert.pfx").unwrap();
    let mut identity = vec![];
    file.read_to_end(&mut identity).unwrap();
    let identity = Identity::from_pkcs12(&identity, "hunter2").unwrap();
    let acceptor = TlsAcceptor::new(identity).unwrap();
    let acceptor = Arc::new(acceptor);

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let acceptor = acceptor.clone();
                pool.execute(move || {
                    let stream = acceptor.accept(stream).unwrap();
                    handle_connection(stream);
                });
            }
            Err(error) => panic!("Problem opening the file: {:?}", error),
        }
    }
    Ok(())
}
