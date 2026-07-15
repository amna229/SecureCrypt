mod client;
mod server;
mod crypto;


fn main() {
    
    println!("Selecciona una opción:\n1.Clásico\n2.Post-Cuántico");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Error al leer la entrada");

    let mode = match input.trim() {
        "1" => crypto::crypto_mode::CryptoMode::Classical,
        "2" => crypto::crypto_mode::CryptoMode::PostQuantum,
        _ => {
            println!("Opción inválida. Se seleccionará el modo clásico por defecto");
            crypto::crypto_mode::CryptoMode::Classical
        }
    };

    let crypto_provider = crypto::crypto_selector::get_crypto_provider(mode);

    let config = crypto_provider.build_client_config().unwrap();

    println!("Configuración creada correctamente");

}
