use tokio::net::TcpListener;
use axum::{
    http::StatusCode, 
    response::IntoResponse, 
    routing::{get, post}, 
    Json, 
    Router
};
use rusqlite::Connection;
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;
use std::fs;

#[tokio::main]
async fn main(){

    if let Err(_e) =  fs::File::open("./data.db3") {

        let con: Connection = rusqlite::Connection::open("./data.db3").unwrap();

        let sql: &str = "CREATE TABLE posts (
            id    INTEGER PRIMARY KEY,
            title  TEXT NOT NULL,
            post_body  TEXT NOT NULL,
            image_file TEXT,
            created_time TIMESTAMP
        )";

        con.execute(sql, []).unwrap();
    }

    let app: Router = Router::new()
        .nest_service("/images", ServeDir::new("images"))
        .route("/get_posts", get(get_posts))
        .route("/create_post", post(create_post))
        .layer(CorsLayer::permissive());

    let listener: TcpListener = TcpListener::bind("127.0.0.1:8081").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn get_posts() -> impl IntoResponse {
    let con: Connection = rusqlite::Connection::open("./data.db3").unwrap();

    let mut stm = con.prepare("SELECT id, title, post_body, image_file, created_time FROM posts ORDER BY id DESC").unwrap();

    let rows = stm.query_map([], |row| {
        Ok(Post {
            id : row.get(0).unwrap(),
            title : row.get(1).unwrap(),
            post_body : row.get(2).unwrap(),
            image_file : row.get(3).unwrap(),
            created_time : row.get(4).unwrap(),
        })
    }).unwrap();

    let mut posts : Vec<Post> = Vec::new();

    for row in rows {
        let post = row.unwrap();

        posts.push(post);
    }

    Json(posts)
}

async fn create_post(Json(post) : Json<PostEntry>) -> impl IntoResponse {

    let con: Connection = Connection::open("./data.db3").unwrap();

    if let Some(image_data) = post.image_data {

        let image_id: uuid::Uuid = uuid::Uuid::new_v4();

        let file_name: String = format!("{}.png", image_id);

        let file_path: String = format!("./images/{}.png", image_id);

        std::fs::write(&file_path, image_data).unwrap();

        let sql = "
            INSERT INTO posts (
                title,
                post_body,
                image_file,
                created_time
            )
            VALUES (
                ?1,
                ?2,
                ?3,
                DATE('now')
        )";

        con.execute(sql, [&post.title, &post.post_body, &file_name]).unwrap();

    }else{
        let sql = "
            INSERT INTO posts (
                title,
                post_body,
                created_time
            )
            VALUES (
                ?1,
                ?2,
                DATE('now')
        )";

        con.execute(sql, [&post.title, &post.post_body]).unwrap();
    }

    StatusCode::OK
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Post {
    id : u32,
    title : String,
    post_body : String,
    image_file : Option<String>,
    created_time : String
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PostEntry {
    title : String,
    post_body : String,
    image_data : Option<Vec<u8>>
}
