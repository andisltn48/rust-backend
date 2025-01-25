use core::fmt;

use actix_web::{delete, get, http::StatusCode, middleware::Logger, post, put, web, App, HttpResponse, HttpServer, Responder, ResponseError};
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
pub struct ApiError {
    pub error_type: String,
    pub message: String,
}

impl ApiError {
    pub fn internal_error(message: String) -> Self {
        ApiError {
            error_type: "InternalError".to_string(),
            message,
        }
    }

    pub fn validation_error(message: String) -> Self {
        ApiError {
            error_type: "ValidationError".to_string(),
            message,
        }
    }

    pub fn not_found(message: String) -> Self {
        ApiError {
            error_type: "NotFound".to_string(),
            message,
        }
    }

    pub fn database_error(message: String) -> Self {
        ApiError {
            error_type: "DatabaseError".to_string(),
            message,
        }
    }
}

// Implementing the ResponseError trait for ApiError
impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self.error_type.as_str() {
            "InternalError" => StatusCode::INTERNAL_SERVER_ERROR,
            "ValidationError" => StatusCode::BAD_REQUEST,
            "NotFound" => StatusCode::NOT_FOUND,
            "DatabaseError" => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR, // Default to InternalServerError
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(ErrorApiResponse { error: self.message.clone() }) // Serializing ApiError to JSON
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

//API Response
#[derive(Deserialize, Serialize)]
pub struct ApiResponse<T> {
    data: T
}
#[derive(Deserialize, Serialize)]
pub struct ErrorApiResponse<T> {
    error: T
}

//Routes
async fn index_page() -> &'static str {
    "Hello world!"
}

#[post("/blog")]
async fn create_blog_post(data: web::Data<PgPool>, new_post: web::Json<NewBlogPost>) -> Result<impl Responder, ApiError> {
    let post = save_blog_post(&data, &new_post).await.expect("failed to save blog post");
    let response = ApiResponse {
        data: post
    };
    Ok(HttpResponse::Ok().json(response))
}

#[get("/blog")]
async fn get_all_blog_posts(data: web::Data<PgPool>) -> Result<impl Responder, ApiError> {
    match find_all_post(&data).await {
        Ok(posts) => {
            let response = ApiResponse {
                data: posts
            };
            Ok(HttpResponse::Ok().json(response))
        },
        Err(err) => return Err(ApiError {
            error_type: "InternalError".to_string(),
            message: err.to_string(),
        })
    }
}

#[get("/blog/{id}")]
async fn get_blog_post_by_id(data: web::Data<PgPool>, id: web::Path<i32>) -> Result<impl Responder, ApiError> {
    match find_post_by_id(&data, id.into_inner()).await {
        Ok(post) => {
            let response = ApiResponse {
                data: post
            };
            Ok(HttpResponse::Ok().json(response))
        },
        Err(err) => {
            if let sqlx::Error::RowNotFound = err {
                return Err(ApiError { error_type: "NotFound".to_string(), message: "blog post not found".to_string() });
            }
            Err(ApiError {
                error_type: "InternalError".to_string(),
                message: err.to_string(),
            })
        }
    }
}

#[put("/blog/{id}")]
async fn update_post(
    data: web::Data<PgPool>,
    id: web::Path<i32>,
    new_post: web::Json<NewBlogPost>
) -> Result<impl Responder, ApiError> {
    match update_post_by_id(&data, id.into_inner(), &new_post).await {
        Ok(post) => {
            let response = ApiResponse {
                data: post
            };
            Ok(HttpResponse::Ok().json(response))
        },
        Err(err) => {
            if let sqlx::Error::RowNotFound = err {
                return Err(ApiError { error_type: "NotFound".to_string(), message: "blog post not found".to_string() });
            }
            Err(ApiError{
                error_type: "InternalError".to_string(),
                message: err.to_string(),
            })
        }
    }
}

#[delete("/blog/{id}")]
async fn delete_post(data: web::Data<PgPool>, id: web::Path<i32>) -> Result<impl Responder, ApiError> {
    match delete_post_by_id(&data, id.into_inner()).await {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(err) => {
            if let sqlx::Error::RowNotFound = err {
                return Err(ApiError { error_type: "NotFound".to_string(), message: "blog post not found".to_string() });
            }
            Err(ApiError{
                error_type: "InternalError".to_string(),
                message: err.to_string(),
            })
        }
        
    }
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
            .service(get_all_blog_posts)
            .service(get_blog_post_by_id)
            .service(update_post)
            .service(delete_post)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
