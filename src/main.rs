mod client;
mod crypto;
mod server;

use std::error::Error;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    println!("Selecciona una opción:\n1.Clásico\n2.Post-Cuántico");

    let mut input = String::new();

    std::io::stdin()
        .read_line(&mut input)
        .expect("Error al leer la entrada");


    let mode = match input.trim() {

        "1" => crypto::crypto_mode::CryptoMode::Classical,

        "2" => crypto::crypto_mode::CryptoMode::PostQuantum,

        _ => {
            println!("Opción inválida. Se seleccionará el modo clásico por defecto");
            crypto::crypto_mode::CryptoMode::Classical
        }
    };


    let crypto_provider =
        crypto::crypto_selector::get_crypto_provider(mode);


    println!("Intentando establecer conexión TLS...");


    let tls_stream =
        client::establish_tls_connection(
            crypto_provider,
            "github.com:443",
            "github.com"
        )
        .await?;


    println!("Conexión TLS establecida correctamente");


    let (_, session) = tls_stream.get_ref();


    println!(
        "Cipher suite negociada: {:?}",
        session.negotiated_cipher_suite()
    );


    // Intentar mostrar el grupo de intercambio de claves negociado
    println!(
        "KX group negociado: {:?}",
        session.negotiated_key_exchange_group()
    );


    Ok(())
}