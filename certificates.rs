use axum::{
extract::{Path, State},
http::StatusCode,
Json,
};
use sqlx::SqlitePool;
use uuid::Uuid;
use chrono::Utc;
use shared::Certificate;

pub async fn issue_certificate(
Path(course_id): Path<Uuid>,
State(pool): State<SqlitePool>,
) -> Result<Json<Certificate>, (StatusCode, String)> {

let user_row = sqlx::query_as::<_, (String,)>("SELECT id FROM users LIMIT 1")
.fetch_one(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

let user_id = user_row.0;

let exists: i64 = sqlx::query_scalar(
"SELECT COUNT(*) FROM certificates WHERE user_id = $1 AND course_id = $2"
)
.bind(&user_id)
.bind(course_id.to_string())
.fetch_one(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

if exists > 0 {
return Err((StatusCode::BAD_REQUEST, "Certificate already issued".to_string()));
}

let certificate_id = Uuid::new_v4();
let now = Utc::now();
let certificate_code = format!("CERT-{}-{}", now.format("%Y%m%d"), Uuid::new_v4().simple());

sqlx::query(
"INSERT INTO certificates (id, user_id, course_id, issued_at, certificate_code)
VALUES ($1, $2, $3, $4, $5)"
)
.bind(certificate_id.to_string())
.bind(&user_id)
.bind(course_id.to_string())
.bind(now.to_rfc3339())
.bind(&certificate_code)
.execute(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

let certificate = Certificate {
id: certificate_id,
user_id: Uuid::parse_str(&user_id).unwrap(),
course_id,
issued_at: now,
certificate_code,
};

Ok(Json(certificate))
}

pub async fn get_my_certificates(
State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Certificate>>, (StatusCode, String)> {

let user_row = sqlx::query_as::<_, (String,)>("SELECT id FROM users LIMIT 1")
.fetch_one(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

let rows = sqlx::query_as::<_, (String, String, String, String, String)>(
"SELECT id, user_id, course_id, issued_at, certificate_code FROM certificates WHERE user_id = $1"
)
.bind(&user_row.0)
.fetch_all(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

let certificates = rows.into_iter().map(|row| Certificate {
id: Uuid::parse_str(&row.0).unwrap(),
user_id: Uuid::parse_str(&row.1).unwrap(),
course_id: Uuid::parse_str(&row.2).unwrap(),
issued_at: chrono::DateTime::parse_from_rfc3339(&row.3).unwrap().with_timezone(&chrono::Utc),
certificate_code: row.4,
}).collect();

Ok(Json(certificates))
}
