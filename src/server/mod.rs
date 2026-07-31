use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use crate::crypto::algorithm_provider::AlgorithmProvider;
use tokio_util::sync::CancellationToken;
use tokio::io::{AsyncReadExt, AsyncWriteExt};


pub async fn run_server(provider: Box<dyn AlgorithmProvider>, addr: &str, cancellation_token: CancellationToken, selected_cipher_suites: &[String], selected_kx_groups: &[String]) -> Result<(), Box<dyn std::error::Error>> {

    let config = provider.build_server_config(selected_cipher_suites, selected_kx_groups)?;

    let tls_acceptor = TlsAcceptor::from(Arc::new(config));

    let listener = TcpListener::bind(addr).await?;

    println!("tfg-project-server up on {}", addr);


    loop {

    tokio::select! {

        _ = cancellation_token.cancelled() => {

            println!("Server shutting down..");

            break Ok(());

        }


        result = listener.accept() => {

            let (stream, client_addr) = result?;

            let acceptor = tls_acceptor.clone();


            tokio::spawn(async move {

                match acceptor.accept(stream).await {

                    Ok(mut tls_stream) => {

                        println!("Server: TLS connection established with {}", client_addr);

                        let mut buffer = [0u8; 1024];

                        loop{

                            let bytes_read = tls_stream.read(&mut buffer).await;

                            match bytes_read {

                                Ok(bytes_read) => {

                                    if bytes_read == 0 {

                                        println!("Server: Connection closed by client {}", client_addr);

                                        break;

                                    } else {

                                        println!("Server: Received {} bytes from client {}", bytes_read, client_addr);

                                        let msg = &buffer[..bytes_read];

                                        if msg == b"PING"{

                                            println!("Server: Received PING from {}", client_addr);

                                            if let Err(e) = tls_stream.write_all(b"PONG").await {

                                                eprintln!("Error sending PONG to client {}: {}", client_addr, e);

                                                break;

                                            }

                                            println!("Server: Sent PONG to client {}", client_addr);

                                        }                                        

                                    }

                                }

                                Err(e) => {

                                    eprintln!("Error reading from TLS stream: {}", e);

                                    break;

                                }

                            }


                        }

                    }


                    Err(e) => {

                        eprintln!("Error accepting TLS connection: {}", e);

                    }

                }

            });

        }

    }

}

}