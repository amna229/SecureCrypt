use axum::{
    extract::Json,
    response::Html,
};
use crate::application::config::ApplicationConfig;



pub async fn my_app() -> Html<&'static str> {

    Html(include_str!("ui/templates/myApp.html"))

}



pub async fn start_app(Json(config): Json<ApplicationConfig>,){

    println!("{:#?}", config);

}