use crate::dashboard::services::manager::Manager;
use crate::dashboard::state::{ClientConfig, DashboardState, ServerConfig};
use crate::dashboard::ui::templates;
use axum::{Json, extract::State, response::Html};
use std::sync::Arc;



pub async fn dashboard() -> Html<&'static str> {

    Html(templates::DASHBOARD)
}



pub async fn server() -> Html<&'static str> {

    Html(templates::SERVER)
}



pub async fn client() -> Html<String> {

    let application_url = "https://127.0.0.1:8443";

    let html =
        templates::CLIENT.replace(
            "{{APPLICATION_URL}}",
            application_url
        );

    Html(html)
}



pub async fn results() -> Html<&'static str> {

    Html(templates::RESULTS)
}



pub async fn save_server_config(State(state): State<Arc<DashboardState>>, Json(config): Json<ServerConfig>) {

    if state.is_evaluation_running().await {

        println!(
            "Cannot modify server configuration while an evaluation is running"
        );

        return;
    }

    println!("{:#?}", config);

    let mut server_config =
        state.server_config.lock().await;

    *server_config = Some(config);

    println!("Server configuration saved");
}



pub async fn save_client_config(State(state): State<Arc<DashboardState>>, Json(config): Json<ClientConfig>) {

    if state.is_evaluation_running().await {

        println!(
            "Cannot modify client configuration while an evaluation is running"
        );

        return;
    }

    println!("{:#?}", config);

    let mut client_configs =
        state.client_configs.lock().await;

    client_configs.clear();
    client_configs.push(config);

    println!(
        "Client configuration saved. Total configurations: {}",
        client_configs.len()
    );
}



pub async fn client_status(State(state): State<Arc<DashboardState>>,) -> Json<bool> {

    let clients =
        state.client_configs.lock().await;

    Json(!clients.is_empty())
}



pub async fn application_status(State(state): State<Arc<DashboardState>>,) -> Json<bool> {

    let running =
        state.application_running.lock().await;

    Json(*running)
}



pub async fn start_evaluation(State(state): State<Arc<DashboardState>>){

    println!("Starting evaluation environment...");

    let manager =
        Manager::new(state);

    match manager.start().await {

        Ok(()) => {

            println!(
                "Evaluation environment started successfully"
            );
        }

        Err(error) => {

            eprintln!(
                "Cannot start evaluation environment: {}",
                error
            );
        }
    }
}



pub async fn stop_application(State(state): State<Arc<DashboardState>>) -> Result<(), String> {

    let manager = Manager::new(state);

    manager.stop().await?;

    Ok(())
}