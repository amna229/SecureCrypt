use tokio::net::TcpStream;
use std::error::Error;
use tokio_rustls::TlsConnector;
use std::sync::Arc;
use rustls::pki_types::ServerName;
use crate::crypto::algorithm_provider::AlgorithmProvider;
use tokio::io::AsyncReadExt;



async fn connect_to_server_tcp(addr: &str) -> Result<TcpStream, Box<dyn Error>>{
    
    let stream = TcpStream::connect(addr).await?;

    Ok(stream)
}




pub async fn run_client(provider: Box<dyn AlgorithmProvider>, addr: &str, domain: &str, selected_cipher_suites: &[String], selected_kx_groups: &[String]) -> Result<(), Box<dyn Error>> {

    let config = provider.build_client_config(selected_cipher_suites, selected_kx_groups)?;

    let tls_connector =TlsConnector::from(Arc::new(config));

    let tcp_stream = connect_to_server_tcp(addr).await?;

    let server_name = ServerName::try_from(domain.to_string())?;

    let mut tls_stream = tls_connector.connect(server_name, tcp_stream).await?;

    println!("Client: TLS connection established with {}", addr);

    let mut buffer = [0u8; 1024];

    //bucle que lee datos del servidor hasta que se cierra la conexión
    loop{

        let bytes_read = tls_stream.read(&mut buffer).await?;

        if bytes_read == 0 {

            println!("Client: Connection closed by server");

            break;

        }   

    }

    Ok(())
}











// pub async fn establish_tls_connection(provider: Box<dyn AlgorithmProvider>, addr: &str, domain: &str, selected_cipher_suites: &[String], selected_kx_groups: &[String], num_connections: u32) -> Result<Vec<TlsStream<TcpStream>>, Box<dyn Error>> {

//     let config = provider.build_client_config(selected_cipher_suites, selected_kx_groups)?;

//     let tls_connector = TlsConnector::from(Arc::new(config));

//     //let tcp_stream = connect_to_server_tcp(addr).await?;

//     //let server_name = ServerName::try_from(domain.to_string())?;

//     //let tls_stream = tls_connector.connect(server_name, tcp_stream).await?;


//     let mut connections = Vec::new();

//     for _ in 0..num_connections {

//         let tcp_stream = connect_to_server_tcp(addr).await?;

//         let server_name = ServerName::try_from(domain.to_string())?;

//         let tls_stream = tls_connector.connect(server_name, tcp_stream).await?;

//         connections.push(tls_stream);
//     }

//     Ok(connections)

// }