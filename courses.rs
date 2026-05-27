use axum::{
extract::{Path, State},
http::StatusCode,
Json,
};
use sqlx::SqlitePool;
use uuid::Uuid;
use chrono::Utc;
use shared::{Course, Lesson, Article, ManualEnrollmentRequest};

pub async fn list_courses(
State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Course>>, (StatusCode, String)> {
let rows = sqlx::query_as::<_, (String, String, String, String, i64, i32, String, String, i64, String)>(
"SELECT id, title, description, content, price, duration_hours, level, image_url, is_published, created_at 
FROM courses WHERE is_published = 1"
)
.fetch_all(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

let courses = rows.into_iter().map(|row| Course {
id: Uuid::parse_str(&row.0).unwrap(),
title: row.1,
description: row.2,
content: row.3,
price: row.4,
duration_hours: row.5,
level: row.6,
image_url: row.7,
is_published: row.8 == 1,
created_at: chrono::DateTime::parse_from_rfc3339(&row.9).unwrap().with_timezone(&chrono::Utc),
}).collect();

Ok(Json(courses))
}

pub async fn get_course(
Path(id): Path<Uuid>,
State(pool): State<SqlitePool>,
) -> Result<Json<Course>, (StatusCode, String)> {
let row = sqlx::query_as::<_, (String, String, String, String, i64, i32, String, String, i64, String)>(
"SELECT id, title, description, content, price, duration_hours, level, image_url, is_published, created_at 
FROM courses WHERE id = $1 AND is_published = 1"
)
.bind(id.to_string())
.fetch_one(&pool)
.await
.map_err(|_| (StatusCode::NOT_FOUND, "Course not found".to_string()))?;

let course = Course {
id: Uuid::parse_str(&row.0).unwrap(),
title: row.1,
description: row.2,
content: row.3,
price: row.4,
duration_hours: row.5,
level: row.6,
image_url: row.7,
is_published: row.8 == 1,
created_at: chrono::DateTime::parse_from_rfc3339(&row.9).unwrap().with_timezone(&chrono::Utc),
};

Ok(Json(course))
}

pub async fn get_course_lessons(
Path(course_id): Path<Uuid>,
State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Lesson>>, (StatusCode, String)> {
let rows = sqlx::query_as::<_, (String, String, String, String, String, i32, i32)>(
"SELECT id, course_id, title, video_url, content, duration_minutes, order_index 
FROM lessons WHERE course_id = $1 ORDER BY order_index"
)
.bind(course_id.to_string())
.fetch_all(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

let lessons = rows.into_iter().map(|row| Lesson {
id: Uuid::parse_str(&row.0).unwrap(),
course_id: Uuid::parse_str(&row.1).unwrap(),
title: row.2,
video_url: row.3,
content: row.4,
duration_minutes: row.5,
order_index: row.6,
}).collect();

Ok(Json(lessons))
}

pub async fn enroll_manual(
State(pool): State<SqlitePool>,
Json(req): Json<ManualEnrollmentRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
let user_row = sqlx::query_as::<_, (String,)>(
"SELECT id FROM users WHERE email = $1"
)
.bind(&req.user_email)
.fetch_one(&pool)
.await
.map_err(|_| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

let user_id = user_row.0;
let enrollment_id = Uuid::new_v4();
let now = Utc::now();

sqlx::query(
"INSERT INTO enrollments (id, user_id, course_id, enrolled_at, is_active)
VALUES ($1, $2, $3, $4, 1)
ON CONFLICT(user_id, course_id) DO NOTHING"
)
.bind(enrollment_id.to_string())
.bind(&user_id)
.bind(req.course_id.to_string())
.bind(now.to_rfc3339())
.execute(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

Ok(StatusCode::OK)
}

pub async fn get_my_courses(
State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Course>>, (StatusCode, String)> {

let rows = sqlx::query_as::<_, (String, String, String, String, i64, i32, String, String, i64, String)>(
"SELECT c.id, c.title, c.description, c.content, c.price, c.duration_hours, c.level, c.image_url, c.is_published, c.created_at
FROM courses c
JOIN enrollments e ON e.course_id = c.id
WHERE e.is_active = 1"
)
.fetch_all(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

let courses = rows.into_iter().map(|row| Course {
id: Uuid::parse_str(&row.0).unwrap(),
title: row.1,
description: row.2,
content: row.3,
price: row.4,
duration_hours: row.5,
level: row.6,
image_url: row.7,
is_published: row.8 == 1,
created_at: chrono::DateTime::parse_from_rfc3339(&row.9).unwrap().with_timezone(&chrono::Utc),
}).collect();

Ok(Json(courses))
}

pub async fn list_articles(
State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Article>>, (StatusCode, String)> {
let rows = sqlx::query_as::<_, (String, String, String, String, i32, String)>(
"SELECT id, title, content, author_id, views, created_at FROM articles ORDER BY created_at DESC"
)
.fetch_all(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

let articles = rows.into_iter().map(|row| Article {
id: Uuid::parse_str(&row.0).unwrap(),
title: row.1,
content: row.2,
author_id: Uuid::parse_str(&row.3).unwrap(),
views: row.4,
created_at: chrono::DateTime::parse_from_rfc3339(&row.5).unwrap().with_timezone(&chrono::Utc),
}).collect();

Ok(Json(articles))
}

pub async fn get_article(
Path(id): Path<Uuid>,
State(pool): State<SqlitePool>,
) -> Result<Json<Article>, (StatusCode, String)> {
sqlx::query("UPDATE articles SET views = views + 1 WHERE id = $1")
.bind(id.to_string())
.execute(&pool)
.await
.ok();

let row = sqlx::query_as::<_, (String, String, String, String, i32, String)>(
"SELECT id, title, content, author_id, views, created_at FROM articles WHERE id = $1"
)
.bind(id.to_string())
.fetch_one(&pool)
.await
.map_err(|_| (StatusCode::NOT_FOUND, "Article not found".to_string()))?;

let article = Article {
id: Uuid::parse_str(&row.0).unwrap(),
title: row.1,
content: row.2,
author_id: Uuid::parse_str(&row.3).unwrap(),
views: row.4,
created_at: chrono::DateTime::parse_from_rfc3339(&row.5).unwrap().with_timezone(&chrono::Utc),
};

Ok(Json(article))
}

pub async fn add_comment(
Path(article_id): Path<Uuid>,
State(pool): State<SqlitePool>,
Json(content): Json<String>,
) -> Result<StatusCode, (StatusCode, String)> {
let comment_id = Uuid::new_v4();
let now = Utc::now();

let user_row = sqlx::query_as::<_, (String,)>("SELECT id FROM users LIMIT 1")
.fetch_one(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

sqlx::query(
"INSERT INTO comments (id, article_id, user_id, content, created_at) VALUES ($1, $2, $3, $4, $5)"
)
.bind(comment_id.to_string())
.bind(article_id.to_string())
.bind(&user_row.0)
.bind(content)
.bind(now.to_rfc3339())
.execute(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

Ok(StatusCode::CREATED)
}
