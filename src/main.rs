
use axum::{
    http::StatusCode, routing::{get, Router},
    response::{Html, IntoResponse},
    extract::{State, Path},
};
use sqlx::{
    postgres::PgPoolOptions,
    FromRow,
    types::time::Date,
};
use std::sync::Arc;
use askama::Template;
use tower_http::services::ServeDir;

mod filters {
    pub fn rmdashes(title: &str) -> askama::Result<String> {
        Ok(title.replace("-", " ").into())
     }
 }

#[derive(Template)]
#[template(path = "posts.html")]
pub struct PostTemplate<'a> {
    pub post_title: &'a str,
    pub post_date: String,
    pub post_body: &'a str,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub index_title: String,
    pub index_links: &'a Vec<String>,
}


#[derive(FromRow, Debug, Clone)]
pub struct Post {
    pub post_title: String,
    pub post_date: Date,
    pub post_body: String,
}

async fn index(State(state): State<Arc<Vec<Post>>>) -> impl IntoResponse{

    let s = state.clone();
    let mut plinks: Vec<String> = Vec::new();

    for i in 0 .. s.len() {
        plinks.push(s[i].post_title.clone());
    }

    let template = IndexTemplate{index_title: String::from("My blog"), index_links: &plinks};

    match template.render() {
            Ok(html) => Html(html).into_response(),
         Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error {}", err),
            ).into_response(),
    }
}

async fn post(Path(query_title): Path<String>, State(state): State<Arc<Vec<Post>>>) -> impl IntoResponse {
    let mut template = PostTemplate {
        post_title: "none",
        post_date: "none".to_string(),
        post_body: "none",
    };
    for i in 0..state.len() {
        if query_title == state[i].post_title {
            template = PostTemplate {
                post_title: &state[i].post_title,
                post_date: state[i].post_date.to_string(),
                post_body: &state[i].post_body
            };
            break;
        } else {
            continue
        }
    }
    if &template.post_title == &"none" {
        return (StatusCode::NOT_FOUND, "404 not found").into_response();
    }
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "try again later").into_response()
    }
}



#[tokio::main]
async fn main() {
    //create pool postgres
    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect("postgres://myuser:mypass@localhost:5432/mydb")
        .await
        .expect("Couldn't connect to the database");
//fetch all posts from db
    let mut posts = sqlx::query_as::<_, Post>("select post_title, post_date, post_body from myposts")
        .fetch_all(&pool)
        .await
        .unwrap();

    for post in &mut posts {
            post.post_title = post.post_title.replace(" ", "-");
         }
//safe sharing data (referencing) to all thread
    let share_state = Arc::new(posts);

    let app = Router::new()
        .route("/", get(index))
        .route("/post/:query_title", get(post))
        .with_state(share_state)
        .nest_service("/assets", ServeDir::new("assets"));

    
         
axum::Server::bind(&"127.0.0.1:6868".parse().unwrap())
    .serve(app.into_make_service())
    .await
    .unwrap();
}