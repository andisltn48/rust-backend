use actix_web::{middleware::Logger, web, App, HttpServer, Responder, HttpResponse, get, post, put, delete};
use env_logger;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};


//Create DB Fxn for connection
pub async fn establish_connection() -> Result<PgPool, sqlx::Error> {
    let database_url = "postgres://postgres:Balikpapan123@localhost:5432/rustdb";
    PgPoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await
}

//Models
#[derive(Debug, Serialize, Deserialize)]
struct BlogPost {
    id: i32,
    title: String,
    content: String,
    author: String
}

//Input Serializer
#[derive(Debug, Serialize, Deserialize)]
struct NewBlogPost {
    id: i32,
    title: String,
    content: String,
    author: String
}

//Routes
//CRUD routes
async fn index_page() -> &'static str {
    "Hello world!"
}

// async fn create_blog_post() -> impl Responder {
    
// }

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
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
