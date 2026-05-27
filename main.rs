mod auth;
mod courses;
mod admin;
mod certificates;
mod db;

use axum::{
http::{HeaderValue, Method},
routing::{get, post, put, delete},
Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use sqlx::sqlite::SqlitePoolOptions;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

let data_dir = Path::new("./data");
if !data_dir.exists() {
std::fs::create_dir_all(data_dir)?;
}


let db_path = data_dir.join("edu.db");
let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

println!("📁 Database path: {}", db_path.display());

let pool = SqlitePoolOptions::new()
.max_connections(5)
.connect(&db_url)
.await?;

db::init_database(&pool).await?;

let cors = CorsLayer::new()
.allow_origin("http://localhost:3000".parse::<HeaderValue>()?)
.allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]);

let app = Router::new()
.route("/api/auth/register", post(auth::register))
.route("/api/auth/login", post(auth::login))
.route("/api/courses", get(courses::list_courses))
.route("/api/courses/:id", get(courses::get_course))
.route("/api/courses/:id/lessons", get(courses::get_course_lessons))
.route("/api/courses/:id/enroll", post(courses::enroll_manual))
.route("/api/my-courses", get(courses::get_my_courses))
.route("/api/articles", get(courses::list_articles))
.route("/api/articles/:id", get(courses::get_article))
.route("/api/articles/:id/comments", post(courses::add_comment))
.route("/api/certificates/my", get(certificates::get_my_certificates))
.route("/api/certificates/:course_id", post(certificates::issue_certificate))
.route("/api/admin/courses", post(admin::create_course))
.route("/api/admin/courses/:id", put(admin::update_course))
.route("/api/admin/courses/:id", delete(admin::delete_course))
.route("/api/admin/lessons", post(admin::add_lesson))
.route("/api/admin/enroll", post(admin::manual_enroll))
.route("/api/admin/users", get(admin::list_users))
.layer(cors)
.layer(TraceLayer::new_for_http())
.with_state(pool);

let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
println!("✅ Server running on http://0.0.0.0:3001");
axum::serve(listener, app).await?;

Ok(())
}
