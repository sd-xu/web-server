use web_server::ThreadPool;

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(5);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    // 如果要优雅停机, 在上面循环的incoming后加上.take(2), 限制循环两次
    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024]; // 创建一个 1024 字节的缓冲区
    stream.read(&mut buffer).unwrap(); // 从 TcpStream 中读取原始字节并放入缓冲区中

    let get_index = b"GET / HTTP/1.1\r\n"; // 转换为字节字符串
    let get_hello = b"GET /hello HTTP/1.1\r\n";
    let get_goodbye = b"GET /goodbye HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get_index) {
        ("HTTP/1.1 200 OK", "static/index.html")
    } else if buffer.starts_with(get_hello) {
        ("HTTP/1.1 200 OK", "static/hello.html")
    } else if buffer.starts_with(get_goodbye) {
        ("HTTP/1.1 200 OK", "static/goodbye.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "static/404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
