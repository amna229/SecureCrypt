use crate::crypto::algorithm_provider::AlgorithmProvider;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_rustls::TlsAcceptor;
use tokio_util::sync::CancellationToken;





pub async fn run_server(
    provider: Box<dyn AlgorithmProvider>,
    addr: &str,
    cancellation_token: CancellationToken,
    selected_cipher_suites: &[String],
    selected_kx_groups: &[String],
    ready_sender: oneshot::Sender<()>,
    pool: sqlx::PgPool
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let config = provider.build_server_config(selected_cipher_suites, selected_kx_groups)?;

    let tls_acceptor = TlsAcceptor::from(Arc::new(config));

    let listener = TcpListener::bind(addr).await?;

    println!("tfg-project-server up on {}", addr);

    let _ = ready_sender.send(());

    loop {
        tokio::select! {

            _ = cancellation_token.cancelled() => {

                println!("Server shutting down..");

                break Ok(());

            }


            result = listener.accept() => {

                let (stream, client_addr) = result?;

                let acceptor = tls_acceptor.clone();

                let pool = pool.clone();
                tokio::spawn(async move {

                    match acceptor.accept(stream).await {

                        Ok(tls_stream) => {println!("Server: TLS connection established with {}", client_addr);

                            if let Err(e) = crate::application::http::serve_connection(tls_stream, pool).await{

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
