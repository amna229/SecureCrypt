mod client;
mod crypto;
mod server;
mod dashboard;

use std::error::Error;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {



    dashboard::run_dashboard().await?;

    Ok(())





    /*
    use std::env;
    use std::io;


    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {

        println!("Uso:");
        println!("  cargo run -- server");
        println!("  cargo run -- client");
        println!("  cargo run -- dashboard");

        return Ok(());

    }

    let role = args[1].as_str();



    println!("Selecciona el modo criptográfico:");
    println!("1. Clásico");
    println!("2. Post-Cuántico");

    let mut crypto_input = String::new();

    io::stdin()
        .read_line(&mut crypto_input)
        .expect("Error al leer la entrada");


    let mode = match crypto_input.trim() {

        "1" => crypto::crypto_mode::CryptoMode::Classical,

        "2" => crypto::crypto_mode::CryptoMode::PostQuantum,

        _ => {

            println!(
                "Opción inválida. Se seleccionará el modo clásico por defecto"
            );

            crypto::crypto_mode::CryptoMode::Classical

        }

    };




    let crypto_provider =
        crypto::crypto_selector::get_crypto_provider(mode);



    match role {



        "server" => {

            println!();
            println!("Iniciando servidor TLS...");

            server::run_server(
                crypto_provider,
                "127.0.0.1:8443",
            )
            .await?;

        }


        "client" => {

            println!();
            println!("Iniciando cliente TLS...");

            let tls_stream =
                client::establish_tls_connection(
                    crypto_provider,
                    "127.0.0.1:8443",
                    "localhost",
                )
                .await?;


            println!(
                "Conexión TLS establecida correctamente"
            );


            // Obtenemos la sesión TLS negociada.

            let (_, session) =
                tls_stream.get_ref();


            // Mostramos la cipher suite negociada.

            println!(
                "Cipher suite negociada: {:?}",
                session.negotiated_cipher_suite()
            );


            // Mostramos el grupo de intercambio de claves
            // negociado.

            println!(
                "KX group negociado: {:?}",
                session.negotiated_key_exchange_group()
            );

        }



        _ => {

            println!(
                "Rol inválido: {}",
                role
            );

            println!();
            println!("Usa:");
            println!("  cargo run -- server");
            println!("  cargo run -- client");
            println!("  cargo run -- dashboard");

        }

    }

    Ok(())
    */

}