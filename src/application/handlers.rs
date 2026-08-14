use axum::{
    extract::Json,
    response::Html,
};
use crate::application::config::ApplicationConfig;
use crate::application::file_generator::generate_file;
use axum::extract::State;
use sqlx::PgPool;




pub async fn my_app() -> Html<&'static str> {

    Html(include_str!("ui/templates/myApp.html"))

}



pub async fn start_app(State(pool): State<PgPool>, Json(config): Json<ApplicationConfig>){

    println!("{:#?}", config);

    let file = generate_file(config.file_size, &config.file_size_unit);

    println!("Generated file: {} bytes", file.len());

    let repository = crate::application::db::repository::ApplicationRepository::new(pool);
    repository.save(&config).await.expect("Failed to save application config");

    print!("Application config saved successfully");

}