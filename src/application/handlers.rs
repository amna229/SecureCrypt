use axum::response::Html;


pub async fn home() -> Html<&'static str> {

    Html(include_str!("templates/home.html"))

}