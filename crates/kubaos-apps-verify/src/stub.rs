//! A local HTTP server for the tests, so no test reaches the network: each request gets the answer
//! a closure picks from its path and how many requests came before it.
//!
//! # Owns
//!
//! - [`Stub`], the server, and [`Reply`], what it can answer, outages included.
//!
//! # Not here
//!
//! - What the fetches make of an answer: `fetch.rs`.

use std::io::Read as _;
use std::io::Write as _;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

/// One answer of the stub.
#[derive(Clone, Debug)]
pub enum Reply {
    /// 200 with the body.
    Body(Vec<u8>),
    /// The status with an empty body.
    Status(u16),
    /// 302 to the path.
    Redirect(String),
    /// The connection closed with no answer.
    Close,
    /// No answer for longer than a fetch waits.
    Stall,
}

type Route = dyn Fn(&str, usize) -> Reply + Send + Sync;

/// A server on a loopback port of its own, for as long as the test runs.
#[derive(Debug)]
pub struct Stub {
    port: u16,
    requests: Arc<AtomicUsize>,
}

impl Stub {
    /// A stub answering each request with `route(path, requests before it)`.
    pub fn new(route: impl Fn(&str, usize) -> Reply + Send + Sync + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("binding a loopback port");
        let port = listener.local_addr().expect("the stub's address").port();
        let requests = Arc::new(AtomicUsize::new(0));
        let route: Arc<Route> = Arc::new(route);
        let counter = Arc::clone(&requests);
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let route = Arc::clone(&route);
                let counter = Arc::clone(&counter);
                thread::spawn(move || serve(stream, &*route, &counter));
            }
        });
        Self { port, requests }
    }

    /// A stub answering every request with `reply`.
    pub fn always(reply: Reply) -> Self {
        Self::new(move |_, _| reply.clone())
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }

    /// How many requests it has had.
    pub fn requests(&self) -> usize {
        self.requests.load(Ordering::SeqCst)
    }
}

/// A loopback URL where nothing listens, which refuses the connection.
pub fn refused(path: &str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("binding a loopback port");
    let port = listener.local_addr().expect("the port's address").port();
    drop(listener);
    format!("http://127.0.0.1:{port}{path}")
}

fn serve(mut stream: TcpStream, route: &Route, requests: &AtomicUsize) {
    let mut request = Vec::new();
    let mut buffer = [0; 1024];
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        match stream.read(&mut buffer) {
            Ok(0) | Err(_) => return,
            Ok(read) => request.extend_from_slice(&buffer[..read]),
        }
        // A TLS handshake's first byte, 22: the plain answer fails the client's handshake at once.
        if request.first() == Some(&0x16) {
            let _ = stream.write_all(b"HTTP/1.1 400 Stub\r\nConnection: close\r\n\r\n");
            return;
        }
    }
    let text = String::from_utf8_lossy(&request);
    let path = text.split_whitespace().nth(1).unwrap_or("/").to_owned();
    let nth = requests.fetch_add(1, Ordering::SeqCst);
    let (status, headers, body) = match route(&path, nth) {
        Reply::Body(body) => (200, String::new(), body),
        Reply::Status(status) => (status, String::new(), Vec::new()),
        Reply::Redirect(to) => (302, format!("Location: {to}\r\n"), Vec::new()),
        Reply::Close => return,
        Reply::Stall => {
            thread::sleep(Duration::from_secs(5));
            return;
        }
    };
    let head = format!(
        "HTTP/1.1 {status} Stub\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(&body);
}
