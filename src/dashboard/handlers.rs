use axum::{
    extract::State,
    response::Html,
    Json,
};
use std::sync::Arc;
use crate::dashboard::state::{
    ClientConfig,
    DashboardState,
    ServerConfig,
};
use crate::dashboard::ui::templates;
use crate::dashboard::services::manager::Manager;



pub async fn dashboard() -> Html<&'static str> {

    Html(templates::DASHBOARD)

}



pub async fn server() -> Html<&'static str> {

    Html(templates::SERVER)

}



pub async fn client() -> Html<&'static str> {

    Html(templates::CLIENT)

}



pub async fn results() -> Html<&'static str> {

    Html(templates::RESULTS)

}



pub async fn save_server_config(State(state): State<Arc<DashboardState>>, Json(config): Json<ServerConfig>){

    if state.is_evaluation_running().await {

        println!("Cannot modify server configuration while an evaluation is running");
        return;

    }

    println!("{:#?}", config);

    let mut server_config = state.server_config.lock().await;
    *server_config = Some(config);

    println!("Server configuration saved");

}



pub async fn save_client_config(State(state): State<Arc<DashboardState>>, Json(config): Json<ClientConfig>){

    if state.is_evaluation_running().await {

        println!("Cannot modify client configuration while an evaluation is running");
        return;

    }

    println!("{:#?}", config);

    let mut client_configs = state.client_configs.lock().await;
    client_configs.push(config);

    println!("Client configuration saved. Total configurations: {}", client_configs.len());

}



pub async fn start_evaluation(State(state): State<Arc<DashboardState>>){

    println!("Starting evaluation environment...");

    let manager = Manager::new(state.clone());


    match manager.start().await{

        Ok(()) => {

            println!("Evaluation environment started successfully");

        }

        Err(error) => {

            eprintln!("Cannot start evaluation environment: {}", error);

        }

    }

}