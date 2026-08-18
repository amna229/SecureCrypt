use tokio::net::TcpStream;
use std::convert::Infallible;
use std::error::Error;
use tokio_rustls::TlsConnector;
use std::sync::Arc;
use rustls::pki_types::ServerName;
use crate::crypto::algorithm_provider::AlgorithmProvider;
use crate::client::config::TransferConfig;
use bytes::Bytes;
use http_body_util::{
    BodyExt,
    Full,
    StreamBody,
    combinators::BoxBody,
};
use hyper::{
    Request,
    body::Frame,
};
use hyper_util::rt::TokioIo;
use reqwest::Client;
use std::time::Duration;
use futures_util::stream;



pub mod config;
pub mod file_generator;



type BoxError =
    Box<dyn Error + Send + Sync>;



fn boxed_infallible(
    error: Infallible
) -> BoxError {

    match error {}
}



async fn connect_to_server_tcp(
    addr: &str
) -> Result<TcpStream, Box<dyn Error + Send + Sync>> {

    let stream =
        TcpStream::connect(addr).await?;

    Ok(stream)
}



pub async fn run_client(
    provider: Box<dyn AlgorithmProvider>,
    addr: &str,
    domain: &str,
    selected_cipher_suites: &[String],
    selected_kx_groups: &[String],
    config_url: &str,
    evaluation_started_at: &str
) -> Result<(), Box<dyn Error + Send + Sync>> {

    let config =
        provider.build_client_config(
            selected_cipher_suites,
            selected_kx_groups
        )?;


    let tls_connector =
        TlsConnector::from(
            Arc::new(config)
        );


    let tcp_stream =
        connect_to_server_tcp(addr).await?;


    let server_name =
        ServerName::try_from(
            domain.to_string()
        )?;


    let tls_stream =
        tls_connector
            .connect(
                server_name,
                tcp_stream
            )
            .await?;


    println!(
        "Client: TLS connection established with {}",
        addr
    );


    let io =
        TokioIo::new(
            tls_stream
        );


    let (mut sender, connection) =
        hyper::client::conn::http1::handshake(
            io
        )
        .await?;


    tokio::spawn(async move {

        match connection.await {

            Ok(()) => {

                eprintln!(
                    "HTTP connection closed normally"
                );
            }

            Err(error) => {

                eprintln!(
                    "HTTP connection driver failed: {}",
                    error
                );
            }
        }
    });


    let ca_cert =
        std::fs::read(
            "simplified-pki/rootCA/rootCA.crt"
        )?;


    let cert =
        reqwest::Certificate::from_pem(
            &ca_cert
        )?;


    let http_client =
        Client::builder()
            .add_root_certificate(cert)
            .build()?;


    let mut last_created_at =
        evaluation_started_at
            .parse::<i64>()?;


    loop {

        let latest_url =
            format!(
                "{}?after={}",
                config_url,
                last_created_at
            );


        let response =
            http_client
                .get(&latest_url)
                .send()
                .await?;


        if response.status()
            == reqwest::StatusCode::NOT_FOUND
        {

            tokio::time::sleep(
                Duration::from_secs(1)
            )
            .await;

            continue;
        }


        let transfer_config =
            response
                .error_for_status()?
                .json::<TransferConfig>()
                .await?;


        println!(
            "New transfer configuration: {:#?}",
            transfer_config
        );


        match transfer_config.operation.as_str() {

            "upload" => {

                const CHUNK_SIZE: u64 =
                    1024 * 1024;


                for file_id
                    in 1..=transfer_config.num_files
                {

                    let total_size =
                        transfer_config
                            .file_size_bytes;


                    let chunks =
                        (total_size
                            + CHUNK_SIZE
                            - 1)
                            / CHUNK_SIZE;


                    let body_stream =
                        stream::iter(
                            (0..chunks).map(
                                move |chunk_id| {

                                    let offset =
                                        chunk_id
                                            * CHUNK_SIZE;


                                    let remaining =
                                        total_size
                                            - offset;


                                    let chunk_size =
                                        remaining.min(
                                            CHUNK_SIZE
                                        );


                                    let data =
                                        file_generator
                                            ::generate_file(
                                                chunk_size
                                            );


                                    Ok::<
                                        Frame<Bytes>,
                                        Infallible
                                    >(
                                        Frame::data(
                                            Bytes::from(
                                                data
                                            )
                                        )
                                    )
                                }
                            )
                        );


                    let body:
                        BoxBody<Bytes, BoxError> =
                        StreamBody::new(
                            body_stream
                        )
                        .map_err(
                            boxed_infallible
                        )
                        .boxed();


                    let request =
                        Request::post(
                            "/upload"
                        )
                        .header(
                            "Content-Type",
                            "application/octet-stream"
                        )
                        .header(
                            "Content-Length",
                            total_size
                        )
                        .body(body)?;


                    sender
                        .ready()
                        .await?;


                    println!(
                        "HTTP connection ready for upload"
                    );


                    let response =
                        sender
                            .send_request(
                                request
                            )
                            .await?;


                    let status =
                        response.status();


                    response
                        .into_body()
                        .collect()
                        .await?;


                    if !status
                        .is_success()
                    {

                        return Err(
                            format!(
                                "Upload failed: {}",
                                status
                            )
                            .into()
                        );
                    }


                    println!(
                        "Uploaded file {}/{}",
                        file_id,
                        transfer_config
                            .num_files
                    );
                }
            }



            "download" => {

                for file_id
                    in 1..=transfer_config.num_files
                {

                    let body:
                        BoxBody<Bytes, BoxError> =
                        Full::new(
                            Bytes::new()
                        )
                        .map_err(
                            boxed_infallible
                        )
                        .boxed();


                    let request =
                        Request::get(
                            format!(
                                "/download?size={}",
                                transfer_config
                                    .file_size_bytes
                            )
                        )
                        .body(body)?;


                    sender
                        .ready()
                        .await?;


                    println!(
                        "HTTP connection ready for download"
                    );


                    let response =
                        sender
                            .send_request(
                                request
                            )
                            .await?;


                    let status =
                        response.status();


                    if !status
                        .is_success()
                    {

                        response
                            .into_body()
                            .collect()
                            .await?;

                        return Err(
                            format!(
                                "Download failed: {}",
                                status
                            )
                            .into()
                        );
                    }


                    let expected_size =
                        transfer_config
                            .file_size_bytes;


                    let mut received_bytes:
                        u64 = 0;


                    let mut body =
                        response.into_body();


                    while let Some(
                        frame_result
                    ) = body.frame().await
                    {

                        let frame =
                            frame_result?;


                        if let Some(data) =
                            frame.data_ref()
                        {

                            received_bytes +=
                                data.len() as u64;
                        }
                    }


                    if received_bytes
                        != expected_size
                    {

                        return Err(
                            format!(
                                "Expected {} bytes, received {}",
                                expected_size,
                                received_bytes
                            )
                            .into()
                        );
                    }


                    println!(
                        "Downloaded file {}/{}: {} bytes",
                        file_id,
                        transfer_config
                            .num_files,
                        received_bytes
                    );
                }
            }



            _ => {

                eprintln!(
                    "Unknown operation: {}",
                    transfer_config.operation
                );
            }
        }


        last_created_at =
            transfer_config.created_at;
    }
}