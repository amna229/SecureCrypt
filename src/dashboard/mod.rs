use axum::{Router, routing::{get, post}};
use tokio::net::TcpListener;
use axum::response::Html;
use axum::Json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use axum::extract::State;
use std::error::Error;



#[derive(Debug, serde::Deserialize)]
pub struct ServerConfig {

    pub key_exchange: crate::crypto::crypto_mode::CryptoMode,
    pub cipher_suites: Vec<String>,
    pub kx_groups: Vec<String>,

}



#[derive(Debug, serde::Deserialize)]
pub struct ClientConfig {

    pub key_exchange: crate::crypto::crypto_mode::CryptoMode,
    pub cipher_suites: Vec<String>,
    pub kx_groups: Vec<String>,
    pub num_connections: u32,

}



pub struct DashboardState {

    pub server_cancellation_token: Mutex<Option<CancellationToken>>,

}




pub async fn run_dashboard() -> Result<(), Box<dyn Error + Send + Sync>> {
    

    let state = Arc::new(DashboardState {server_cancellation_token:Mutex::new(None)}
);

    let app = Router::new()
    .route("/", get(dashboard))
    .route("/server", get(server))
    .route("/client", get(client))
    .route("/results", get(results))
    .route("/info/server", post(save_server_config))
    .route("/info/client", post(save_client_config))
    .with_state(state);
    


    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    println!("Dashboard at http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())

}



async fn dashboard() -> Html<&'static str> {
    
     Html(include_str!("../../templates/dashboard.html"))

}



async fn server() -> Html<&'static str> {
    
     Html(include_str!("../../templates/server.html"))

}


async fn client() -> Html<&'static str> {
    
     Html(include_str!("../../templates/client.html"))

}


async fn results() -> Html<&'static str> {
    
     Html(include_str!("../../templates/results.html"))

}


async fn save_server_config(State(state): State<Arc<DashboardState>>, Json(config): Json<ServerConfig>) {

    println!("{:#?}", config);


    // Detengo el servidor anterior si existe
    let old_token = {

        let guard = state.server_cancellation_token.lock().await;

        guard.clone()

    };


    if let Some(token) = old_token {

        println!("Stopping previous server...");

        token.cancel();

    }


    //creo un nuevo proveedor de cifrado según la configuración
    let provider = crate::crypto::crypto_selector::get_crypto_provider(config.key_exchange);

    
    //creo un nuevo token de cancelación para el nuevo servidor
    let cancellation_token = tokio_util::sync::CancellationToken::new();


    //guardo el nuevo token de cancelación en el estado compartido
    {

        let mut guard = state.server_cancellation_token.lock().await;

        *guard =  Some(cancellation_token.clone());

    }


    println!("Starting new server...");


    // Extraigo las suites de cifrado y los grupos de intercambio de claves seleccionados
    let selected_cipher_suites = config.cipher_suites.clone();

    let selected_kx_groups = config.kx_groups.clone();


    //lanzo el servidor en un nuevo hilo
    tokio::spawn(async move {

        if let Err(e) =
            crate::server::run_server(
                provider,
                "127.0.0.1:8443",
                cancellation_token,
                &selected_cipher_suites,
                &selected_kx_groups,
            )
            .await
        {

            eprintln!(
                "Error in server: {}",
                e
            );

        }

    });

}



async fn save_client_config(Json(config): Json<ClientConfig>) {

    println!("{:#?}", config);

    println!("Starting {} client(s)...", config.num_connections);



    // Extraigo las suites de cifrado y los grupos de intercambio de claves seleccionados
    let selected_cipher_suites = config.cipher_suites.clone();

    let selected_kx_groups = config.kx_groups.clone();



    // Creo num_connections clientes independientes
    for client_id in 0..config.num_connections {

        // Clono los datos
        let provider = crate::crypto::crypto_selector::get_crypto_provider(config.key_exchange);

        let selected_cipher_suites = selected_cipher_suites.clone();

        let selected_kx_groups = selected_kx_groups.clone();


        // Lanzo cliente independiente
        tokio::spawn(async move {

            println!("Starting client {}...", client_id + 1);


            if let Err(e) = crate::client::run_client(
                    provider,
                    "127.0.0.1:8443",
                    "localhost",
                    &selected_cipher_suites,
                    &selected_kx_groups,
                )
                .await
            {

                eprintln!("Error in client {}: {}", client_id + 1, e);

            }

        });

    }

}


