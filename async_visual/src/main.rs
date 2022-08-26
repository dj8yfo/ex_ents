use async_std::io::prelude::*;
use async_std::net;
use colored::{Color, Colorize};
use std::future::Future;

type LocalRes = std::io::Result<(String, Color)>;
fn helper_print(input: &dyn std::fmt::Display) {
    eprintln!(
        "{:?}:{}",
        std::thread::current().id(),
        input
    )
}
fn cheapo_request<'b, 'a: 'b>(
    host: &'a str,
    port: u16,
    path: &'b str,
    color: Color,
) -> impl Future<Output = LocalRes> + 'static {
    let host = host.to_string();
    let path = path.to_string();
    async move {
        helper_print(&"Awaiting connection retrival...".color(color));

        let mut socket = net::TcpStream::connect((host.as_ref(), port)).await?;
        helper_print(&format!("obtained {}", format!("{:?}", socket).color(color)));

        let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, host);
        helper_print(&"sending request...".color(color));
        socket.write_all(request.as_bytes()).await?;
        socket.shutdown(net::Shutdown::Write)?;
        helper_print(&"request sent".color(color));

        let mut response = String::new();
        helper_print(&"reading response...".color(color));
        socket.read_to_string(&mut response).await?;
        helper_print(&"retrived response".color(color));

        Ok((response, color))
    }
}
fn cheapo_request1<'b, 'a: 'b>(
    host: &'a str,
    port: u16,
    path: &'b str,
    color: Color,
) -> impl Future<Output = LocalRes> + 'b {
    async move {
        eprintln!("{}", "Awaiting connection retrival...".color(color));

        let mut socket = net::TcpStream::connect((host, port)).await?;
        eprintln!("obtained {}", format!("{:?}", socket).color(color));

        let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, host);
        eprintln!("{}", "sending request...".color(color));
        socket.write_all(request.as_bytes()).await?;
        socket.shutdown(net::Shutdown::Write)?;
        eprintln!("{}", "request sent".color(color));

        let mut response = String::new();
        eprintln!("{}", "reading response...".color(color));
        socket.read_to_string(&mut response).await?;
        eprintln!("{}", "retrived response".color(color));

        Ok((response, color))
    }
}

async fn many_requests1(
    requests: Vec<(String, u16, String, Color)>,
) -> Vec<std::io::Result<(String, Color)>> {
    use async_std::task;

    let mut handles = vec![];
    for (host, port, path, color) in requests {
        handles.push(task::spawn_local(async move {
            let res = cheapo_request1(&host, port, &path, color).await?;
            Ok(res)
        }));
    }

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }

    results
}

async fn many_requests(
    requests: Vec<(String, u16, String, Color)>,
) -> Vec<std::io::Result<(String, Color)>> {
    use async_std::task;

    let mut handles = vec![];
    for (host, port, path, color) in requests {
        handles.push(task::spawn(cheapo_request(&host, port, &path, color)));
    }

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }

    results
}

fn main() {
    let requests = vec![
        (
            "example.com".to_string(),
            80,
            "/".to_string(),
            Color::Yellow,
        ),
        (
            "www.red-bean.com".to_string(),
            80,
            "/".to_string(),
            Color::Magenta,
        ),
        (
            "en.wikipedia.org".to_string(),
            80,
            "/".to_string(),
            Color::Cyan,
        ),
        (
            "en.wikipedia.org".to_string(),
            80,
            "/wiki".to_string(),
            Color::Red,
        ),
    ];

    let results = async_std::task::block_on(many_requests(requests));
    eprintln!();
    eprintln!();
    for result in results {
        match result {
            Ok((_, color)) => eprintln!("{}", "response retrived {...}".color(color)),
            Err(err) => eprintln!("error: {}", err),
        }
    }

    println!("current thread name {:?} ", std::thread::current().id())
}
