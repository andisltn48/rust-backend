use core::fmt;

use actix_web::{delete, get, middleware::Logger, post, put, web, App, HttpResponse, HttpServer, Responder, ResponseError};
use env_logger;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, prelude::FromRow, PgPool};


//Create DB Fxn for connection
pub async fn establish_connection() -> Result<PgPool, sqlx::Error> {
    let database_url = "postgres://postgres:Balikpapan123@localhost:5432/rustdb";
    PgPoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await
}

//Models
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BlogPost {
    id: i32,
    title: String,
    content: String,
    author: String
}

//Input Serializer
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NewBlogPost {
    title: String,
    content: String,
    author: String
}

//SQLx functions
pub async fn save_blog_post(pool: &PgPool, blog_post: &NewBlogPost) -> Result<BlogPost, sqlx::Error> {
    let post = sqlx::query_as::<_, BlogPost>("INSERT INTO blog_posts (title, content, author) VALUES ($1, $2, $3) RETURNING *")
    .bind(&blog_post.title)
    .bind(&blog_post.content)
    .bind(&blog_post.author)
    .fetch_one(pool)
    .await?;
    Ok(post)
}

pub async fn find_all_post(pool: &PgPool) -> Result<Vec<BlogPost>, sqlx::Error> {
    let posts = sqlx::query_as::<_, BlogPost>("SELECT * FROM blog_posts")
    .fetch_all(pool)
    .await?;
    Ok(posts)
}

pub async fn find_post_by_id(pool: &PgPool, id: i32) -> Result<BlogPost, sqlx::Error> {
    let post = sqlx::query_as::<_, BlogPost>("SELECT * FROM blog_posts WHERE id = $1")
    .bind(id)
    .fetch_one(pool)
    .await?;
    Ok(post)
}

pub async fn delete_post_by_id(pool: &PgPool, id: i32) -> Result<BlogPost, sqlx::Error> {
    let post = sqlx::query_as::<_, BlogPost>("DELETE FROM blog_posts WHERE id = $1 RETURNING *")
    .bind(id)
    .fetch_one(pool)
    .await?;
    Ok(post)
}

pub async fn update_post_by_id(pool: &PgPool, id: i32, blog_post: &NewBlogPost) -> Result<BlogPost, sqlx::Error> {
    let post = sqlx::query_as::<_, BlogPost>("UPDATE blog_posts SET title = $1, content = $2, author = $3 WHERE id = $4 RETURNING *")
    .bind(&blog_post.title)
    .bind(&blog_post.content)
    .bind(&blog_post.author)
    .bind(id)
    .fetch_one(pool)
    .await?;
    Ok(post)
}

//Error handling
#[derive(Debug, Serialize, Deserialize)]
enum ApiError {
    InternalError(String),
    ValidationError(String),
    NotFound(String),
    DatabaseError(String)
}

impl ResponseError for ApiError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            ApiError::InternalError(msg) => actix_web::HttpResponse::InternalServerError().json(msg),
            ApiError::ValidationError(msg) => actix_web::HttpResponse::BadRequest().json(msg),
            ApiError::NotFound(msg) => actix_web::HttpResponse::NotFound().json(msg),
            ApiError::DatabaseError(msg) => actix_web::HttpResponse::InternalServerError().json(msg)
        }
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ApiError::InternalError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ValidationError(_) => actix_web::http::StatusCode::BAD_REQUEST,
            ApiError::NotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            ApiError::DatabaseError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::InternalError(msg) => write!(f, "Internal Server Error: {}", msg),
            ApiError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            ApiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ApiError::DatabaseError(msg) => write!(f, "Database Error: {}", msg)
        }
    }
}

//Routes
async fn index_page() -> &'static str {
    "Hello world!"
}

#[post("/blog")]
async fn create_blog_post(data: web::Data<PgPool>, new_post: web::Json<NewBlogPost>) -> Result<impl Responder, ApiError> {
    let post = save_blog_post(&data, &new_post).await.expect("failed to save blog post");
    Ok(HttpResponse::Ok().json(post))
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    //Server configuration
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    //Database config
    let pool = establish_connection().await.expect("failed to connect to db");

    HttpServer::new(move|| {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .route("/", web::get().to(index_page))
            .service(create_blog_post)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
