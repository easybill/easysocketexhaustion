use std::net::SocketAddr;
use anyhow::Context;
use clap::{Parser, Subcommand};
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Opts {
    #[arg(long = "ip_listen")]
    ips_listen: Vec<String>,
    #[arg(long = "ip_bench")]
    ips_bench: Vec<String>,
}

#[tokio::main]
async fn main() -> ::anyhow::Result<()> {
    let opts = Opts::parse();

    println!("Hello, world!");

    let mut listens = vec![];
    for listen in opts.ips_listen {
        listens.push(spawn_listen(listen).await.context("could not listen")?)
    }

    let (res, _, _) = ::futures::future::select_all(listens).await;

    Ok(res?)
}

async fn spawn_listen(interface: String) -> Result<JoinHandle<()>, ::anyhow::Error> {

    let listener = TcpListener::bind(interface).await.context("could not bind listener.")?;

    let jh = ::tokio::spawn(async move {
        loop {
            let v = match listener.accept().await {
                Ok((stream, socket)) => {
                    ::tokio::spawn(async move {
                        spawn_listen_client(stream, socket).await;
                    });
                },
                Err(e) => {
                    eprintln!("could not accept connection, reason: {}", e);
                    continue;
                }
            };
        }
    });


    Ok(jh)
}

async fn spawn_listen_client(mut stream: TcpStream, socket: SocketAddr) {
    let (mut r, mut w) = stream.into_split();
    let mut buffer = [0u8; 4096];
    loop {
        let size = match r.read(&mut buffer).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("could not read from connection, reason: {}", e);
                continue;
            }
        };

        match w.write_all(&buffer[0..size]).await {
            Ok(_) => {},
            Err(e) => {
                eprintln!("could not write to connection, reason: {}", e);
                continue;
            }
        };
    }
}
