use actix_web::{middleware::Logger, web, App, HttpServer, Responder, HttpResponse, get, post, put, delete};
use env_logger;
use serde::{Deserialize, Serialize};

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

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    //Server configuration
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/", web::get().to(index_page))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
