use crate::crypto::algorithm_provider::AlgorithmProvider;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_rustls::TlsAcceptor;
use tokio_util::sync::CancellationToken;

pub mod http;
pub mod routes;





pub async fn run_server<F, Fut>(
    provider: Box<dyn AlgorithmProvider>,
    addr: &str,
    cancellation_token: CancellationToken,
    selected_cipher_suites: &[String],
    selected_kx_groups: &[String],
    ready_sender: oneshot::Sender<()>,
    connection_handler: Arc<F>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: Fn(
            tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
        ) -> Fut
        + Send
        + Sync
        + 'static,

    Fut: std::future::Future<
            Output = Result<(), Box<dyn std::error::Error + Send + Sync>>,
        >
        + Send
        + 'static,
{
    let config =
        provider.build_server_config(
            selected_cipher_suites,
            selected_kx_groups,
        )?;

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

                let connection_handler = Arc::clone(&connection_handler);

                tokio::spawn(async move {

                    match acceptor.accept(stream).await {

                        Ok(tls_stream) => {

                            println!("Server: TLS connection established with {}", client_addr);

                            if let Err(e) = connection_handler(tls_stream).await
                            {
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