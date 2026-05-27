use sqlx::{SqlitePool, Result};

pub async fn init_database(pool: &SqlitePool) -> Result<()> {

sqlx::query(
r#"
CREATE TABLE IF NOT EXISTS users (
id TEXT PRIMARY KEY,
email TEXT UNIQUE NOT NULL,
username TEXT UNIQUE NOT NULL,
full_name TEXT NOT NULL,
password_hash TEXT NOT NULL,
is_admin INTEGER NOT NULL DEFAULT 0,
created_at TEXT NOT NULL
)
"#
).execute(pool).await?;


sqlx::query(
r#"
CREATE TABLE IF NOT EXISTS courses (
id TEXT PRIMARY KEY,
title TEXT NOT NULL,
description TEXT NOT NULL,
content TEXT NOT NULL,
price INTEGER NOT NULL,
duration_hours INTEGER NOT NULL,
level TEXT NOT NULL,
image_url TEXT NOT NULL,
is_published INTEGER NOT NULL DEFAULT 0,
created_at TEXT NOT NULL
)
"#
).execute(pool).await?;


sqlx::query(
r#"
CREATE TABLE IF NOT EXISTS lessons (
id TEXT PRIMARY KEY,
course_id TEXT NOT NULL,
title TEXT NOT NULL,
video_url TEXT NOT NULL,
content TEXT NOT NULL,
duration_minutes INTEGER NOT NULL,
order_index INTEGER NOT NULL,
FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE
)
"#
).execute(pool).await?;


sqlx::query(
r#"
CREATE TABLE IF NOT EXISTS enrollments (
id TEXT PRIMARY KEY,
user_id TEXT NOT NULL,
course_id TEXT NOT NULL,
enrolled_at TEXT NOT NULL,
is_active INTEGER NOT NULL DEFAULT 1,
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
UNIQUE(user_id, course_id)
)
"#
).execute(pool).await?;


sqlx::query(
r#"
CREATE TABLE IF NOT EXISTS certificates (
id TEXT PRIMARY KEY,
user_id TEXT NOT NULL,
course_id TEXT NOT NULL,
issued_at TEXT NOT NULL,
certificate_code TEXT UNIQUE NOT NULL,
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE
)
"#
).execute(pool).await?;

sqlx::query(
r#"
CREATE TABLE IF NOT EXISTS articles (
id TEXT PRIMARY KEY,
title TEXT NOT NULL,
content TEXT NOT NULL,
author_id TEXT NOT NULL,
views INTEGER NOT NULL DEFAULT 0,
created_at TEXT NOT NULL,
FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE
)
"#
).execute(pool).await?;


sqlx::query(
r#"
CREATE TABLE IF NOT EXISTS comments (
id TEXT PRIMARY KEY,
article_id TEXT NOT NULL,
user_id TEXT NOT NULL,
content TEXT NOT NULL,
created_at TEXT NOT NULL,
FOREIGN KEY (article_id) REFERENCES articles(id) ON DELETE CASCADE,
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
)
"#
).execute(pool).await?;


let admin_exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_admin = 1")
.fetch_one(pool)
.await?;

if admin_exists == 0 {
let hashed = bcrypt::hash("admin123", bcrypt::DEFAULT_COST).unwrap();
let now = chrono::Utc::now();
sqlx::query(
"INSERT INTO users (id, email, username, full_name, password_hash, is_admin, created_at)
VALUES ($1, $2, $3, $4, $5, 1, $6)"
)
.bind(uuid::Uuid::new_v4().to_string())
.bind("admin@edu.com")
.bind("admin")
.bind("Admin User")
.bind(hashed)
.bind(now.to_rfc3339())
.execute(pool)
.await?;

println!("✅ Admin user created: admin@edu.com / admin123");
}

Ok(())
}
