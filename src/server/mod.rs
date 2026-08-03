use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use crate::crypto::algorithm_provider::AlgorithmProvider;
use tokio_util::sync::CancellationToken;




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

                    Ok(tls_stream) => {println!("Server: TLS connection established with {}", client_addr);

                        if let Err(e) = crate::application::http::serve_connection(tls_stream).await{

                            eprintln!("Error serving HTTP connection for {}: {}", client_addr, e);
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