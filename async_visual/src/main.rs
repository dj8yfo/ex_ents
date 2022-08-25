use async_std::io::prelude::*;
use async_std::net;
use colored::{Color, Colorize};

async fn cheapo_request(host: &str, port: u16, path: &str, color: Color)
                            -> std::io::Result<(String, Color)>
{

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

async fn many_requests(requests: Vec<(String, u16, String, Color)>)
                           -> Vec<std::io::Result<(String, Color)>>
{
    use async_std::task;

    let mut handles = vec![];
    for (host, port, path, color) in requests {
        handles.push(task::spawn_local(async move {
            cheapo_request(&host, port, &path, color).await
        }));
    }

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }

    results
}

fn main() {
    let requests = vec![
        ("example.com".to_string(),      80, "/".to_string(), Color::Yellow),
        ("www.red-bean.com".to_string(), 80, "/".to_string(), Color::Magenta),
        ("en.wikipedia.org".to_string(), 80, "/".to_string(), Color::Cyan),
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
}
