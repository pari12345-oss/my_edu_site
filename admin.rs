use axum::{
extract::{Path, State},
http::StatusCode,
Json,
};
use sqlx::SqlitePool; 
use uuid::Uuid;
use chrono::Utc;
use shared::{CreateCourseRequest, CreateLessonRequest, Course, Lesson, User, ManualEnrollmentRequest};

pub async fn create_course(
State(pool): State<SqlitePool>,
Json(req): Json<CreateCourseRequest>,
) -> Result<Json<Course>, (StatusCode, String)> {
let course_id = Uuid::new_v4();
let now = Utc::now();

sqlx::query(
"INSERT INTO courses (id, title, description, content, price, duration_hours, level, image_url, is_published, created_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 1, $9)"
)
.bind(course_id.to_string())
.bind(&req.title)
.bind(&req.description)
.bind(&req.content)
.bind(req.price)
.bind(req.duration_hours)
.bind(&req.level)
.bind(&req.image_url)
.bind(now.to_rfc3339())
.execute(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

let course = Course {
id: course_id,
title: req.title,
description: req.description,
content: req.content,
price: req.price,
duration_hours: req.duration_hours,
level: req.level,
image_url: req.image_url,
is_published: true,
created_at: now,
};

Ok(Json(course))
}

pub async fn update_course(
Path(id): Path<Uuid>,
State(pool): State<SqlitePool>,
Json(req): Json<CreateCourseRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
sqlx::query(
"UPDATE courses SET title = $1, description = $2, content = $3, price = $4, 
duration_hours = $5, level = $6, image_url = $7 WHERE id = $8"
)
.bind(&req.title)
.bind(&req.description)
.bind(&req.content)
.bind(req.price)
.bind(req.duration_hours)
.bind(&req.level)
.bind(&req.image_url)
.bind(id.to_string())
.execute(&pool)
.await
.map_err(|_| (StatusCode::NOT_FOUND, "Course not found".to_string()))?;

Ok(StatusCode::OK)
}

pub async fn delete_course(
Path(id): Path<Uuid>,
State(pool): State<SqlitePool>,
) -> Result<StatusCode, (StatusCode, String)> {
sqlx::query("DELETE FROM courses WHERE id = $1")
.bind(id.to_string())
.execute(&pool)
.await
.map_err(|_| (StatusCode::NOT_FOUND, "Course not found".to_string()))?;

Ok(StatusCode::OK)
}

pub async fn add_lesson(
State(pool): State<SqlitePool>,
Json(req): Json<CreateLessonRequest>,
) -> Result<Json<Lesson>, (StatusCode, String)> {
let lesson_id = Uuid::new_v4();

sqlx::query(
"INSERT INTO lessons (id, course_id, title, video_url, content, duration_minutes, order_index)
VALUES ($1, $2, $3, $4, $5, $6, $7)"
)
.bind(lesson_id.to_string())
.bind(req.course_id.to_string())
.bind(&req.title)
.bind(&req.video_url)
.bind(&req.content)
.bind(req.duration_minutes)
.bind(req.order_index)
.execute(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

let lesson = Lesson {
id: lesson_id,
course_id: req.course_id,
title: req.title,
video_url: req.video_url,
content: req.content,
duration_minutes: req.duration_minutes,
order_index: req.order_index,
};

Ok(Json(lesson))
}

pub async fn manual_enroll(
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

let enrollment_id = Uuid::new_v4();
let now = Utc::now();

sqlx::query(
"INSERT INTO enrollments (id, user_id, course_id, enrolled_at, is_active)
VALUES ($1, $2, $3, $4, 1)"
)
.bind(enrollment_id.to_string())
.bind(&user_row.0)
.bind(req.course_id.to_string())
.bind(now.to_rfc3339())
.execute(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

Ok(StatusCode::CREATED)
}

pub async fn list_users(
State(pool): State<SqlitePool>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
let rows = sqlx::query_as::<_, (String, String, String, String, i64, String)>(
"SELECT id, email, username, full_name, is_admin, created_at FROM users"
)
.fetch_all(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

let users = rows.into_iter().map(|row| User {
id: Uuid::parse_str(&row.0).unwrap(),
email: row.1,
username: row.2,
full_name: row.3,
is_admin: row.4 == 1,
created_at: chrono::DateTime::parse_from_rfc3339(&row.5).unwrap().with_timezone(&chrono::Utc),
}).collect();

Ok(Json(users))
}
