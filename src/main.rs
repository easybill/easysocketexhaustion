use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::time::Duration;
use anyhow::Context;
use clap::{Parser, Subcommand};
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

static BYTES_RECV_CLIENT : AtomicU64 = AtomicU64::new(0);
static BYTES_RECV_SERVER : AtomicU64 = AtomicU64::new(0);
static ERRORS : AtomicU64 = AtomicU64::new(0);
static CONNECTIONS_SEND : AtomicU64 = AtomicU64::new(0);
static CONNECTIONS_RECV : AtomicU64 = AtomicU64::new(0);

static CONFIG_WAIT_NEW_CLIENT_CONNECTION : AtomicUsize = AtomicUsize::new(0);


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Opts {
    #[arg(long = "ip_listen")]
    ips_listen: Vec<String>,
    #[arg(long = "ip_bench")]
    ips_bench: Vec<String>,
    #[arg(long = "parallel", default_value = "5")]
    parallel: u64,
    #[arg(long = "wait_new_client_microseconds", default_value = "100000")]
    wait_new_client_microseconds: u64,
}

#[tokio::main]
async fn main() -> ::anyhow::Result<()> {
    let opts = Opts::parse();

    CONFIG_WAIT_NEW_CLIENT_CONNECTION.store(opts.wait_new_client_microseconds as usize, SeqCst);

    let mut listens = vec![];
    for listen in opts.ips_listen {
        listens.push(spawn_listen(listen).await.context("could not listen")?)
    }

    for instance in 0..opts.parallel {
        for ip in opts.ips_bench.clone() {
            ::tokio::spawn(async move {
                spawn_bench(instance, ip.to_string()).await;
            });
        }
    }


    ::tokio::spawn(async move {
        loop {
            ::tokio::time::sleep(Duration::from_secs(1)).await;
            println!("");
            println!("");
            println!("ERRORS {}", ERRORS.swap(0, Relaxed));
            println!("CONNECTIONS_SEND {}", CONNECTIONS_SEND.swap(0, Relaxed));
        }
    }).await;

    Ok(())
}

async fn spawn_bench(instance: u64, ip: String) {
    loop {
        let mut outbound = match TcpStream::connect(ip.clone()).await {
            Ok(k) => k,
            Err(e) => {
                ERRORS.fetch_add(1, Relaxed);
                eprintln!("could not connect to {}: error: {}", ip, e);
                continue;
            }
        };

        CONNECTIONS_SEND.fetch_add(1, Relaxed);

        let (mut r, mut w) = outbound.into_split();

        let token = CancellationToken::new();

        let wip = ip.clone();
        let wtoken = token.clone();
        let wjh = ::tokio::spawn(async move {
            loop {
                ::tokio::select! {
                    _ = wtoken.cancelled() => {
                        return;
                    },
                    v = w.write_all("test".as_bytes()) => {
                        match v {
                            Ok(_) => {},
                            Err(e) => {
                                ERRORS.fetch_add(1, Relaxed);
                                eprintln!("could not write to {}, error: {}", wip, e);
                                continue;
                            }
                        };

                        ::tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        });

        let rip = ip.clone();
        let rtoken = token.clone();
        let rjh = ::tokio::spawn(async move {
            let mut buffer = [0u8; 64];
            loop {
                ::tokio::select! {
                     _ = rtoken.cancelled() => {
                        return;
                    },
                    v = r.read(&mut buffer) => {
                        match v {
                            Ok(k) => {

                                if (k == 0) {
                                    return;
                                }

                                BYTES_RECV_CLIENT.fetch_add(k as u64, Relaxed);
                            },
                            Err(e) => {
                                ERRORS.fetch_add(1, Relaxed);
                                eprintln!("could not read to {}, error: {}", rip, e);
                                continue;
                            }
                        };
                    }
                }
            }
        });

        ::tokio::time::sleep(Duration::from_micros(CONFIG_WAIT_NEW_CLIENT_CONNECTION.load(Ordering::Relaxed) as u64)).await;
        token.cancel();
    }
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
                    ERRORS.fetch_add(1, Relaxed);
                    eprintln!("could not accept connection, reason: {}", e);
                    continue;
                }
            };
        }
    });


    Ok(jh)
}

async fn spawn_listen_client(mut stream: TcpStream, socket: SocketAddr) {
    CONNECTIONS_RECV.fetch_add(1, Relaxed);
    let (mut r, mut w) = stream.into_split();
    let mut buffer = [0u8; 4096];
    loop {
        let size = match r.read(&mut buffer).await {
            Ok(v) => {
                if v == 0 {
                    break;
                }

                v
            },
            Err(e) => {
                // ERRORS.fetch_add(1, Relaxed);
                // eprintln!("could not read from connection, reason: {}", e);
                continue;
            }
        };

        BYTES_RECV_SERVER.fetch_add(size as u64, Relaxed);

        match w.write_all(&buffer[0..size]).await {
            Ok(_) => {},
            Err(e) => {
                // ERRORS.fetch_add(1, Relaxed);
                // eprintln!("could not write to connection, reason: {}", e);
                continue;
            }
        };
    }
}
