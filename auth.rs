use axum::{
extract::State,
http::StatusCode,
Json,
};
use sqlx::SqlitePool;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Utc, Duration};
use uuid::Uuid;
use shared::{RegisterRequest, LoginRequest, LoginResponse, User};
use jsonwebtoken::{encode, EncodingKey, Header};

const JWT_SECRET: &[u8] = b"your-secret-key";

fn generate_token(user_id: &str, email: &str, is_admin: bool) -> String {
let claims = serde_json::json!({
"sub": user_id,
"email": email,
"admin": is_admin,
"exp": (Utc::now() + Duration::days(7)).timestamp()
});

encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET)).unwrap()
}

pub async fn register(
State(pool): State<SqlitePool>,
Json(req): Json<RegisterRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
let exists: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1 OR username = $2")
.bind(&req.email)
.bind(&req.username)
.fetch_one(&pool)
.await
{
Ok(count) => count,
Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
};

if exists > 0 {
return Err((StatusCode::BAD_REQUEST, "Email or username already exists".to_string()));
}

let hashed = match hash(&req.password, DEFAULT_COST) {
Ok(h) => h,
Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
};

let user_id = Uuid::new_v4();
let now = Utc::now();

match sqlx::query(
"INSERT INTO users (id, email, username, full_name, password_hash, is_admin, created_at)
VALUES ($1, $2, $3, $4, $5, 0, $6)"
)
.bind(user_id.to_string())
.bind(&req.email)
.bind(&req.username)
.bind(&req.full_name)
.bind(hashed)
.bind(now.to_rfc3339())
.execute(&pool)
.await
{
Ok(_) => (),
Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
}

let user = User {
id: user_id,
email: req.email,
username: req.username,
full_name: req.full_name,
is_admin: false,
created_at: now,
};

let token = generate_token(&user_id.to_string(), &user.email, user.is_admin);

Ok(Json(LoginResponse { token, user }))
}

pub async fn login(
State(pool): State<SqlitePool>,
Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
let row = match sqlx::query_as::<_, (String, String, String, String, String, i64, String)>(
"SELECT id, email, username, full_name, password_hash, is_admin, created_at FROM users WHERE email = $1"
)
.bind(&req.email)
.fetch_one(&pool)
.await
{
Ok(row) => row,
Err(_) => return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string())),
};

let valid = match verify(&req.password, &row.4) {
Ok(v) => v,
Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
};

if !valid {
return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
}

let user = User {
id: match Uuid::parse_str(&row.0) {
Ok(id) => id,
Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
},
email: row.1,
username: row.2,
full_name: row.3,
is_admin: row.5 == 1,
created_at: match chrono::DateTime::parse_from_rfc3339(&row.6) {
Ok(dt) => dt.with_timezone(&chrono::Utc),
Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
},
};

let token = generate_token(&user.id.to_string(), &user.email, user.is_admin);

Ok(Json(LoginResponse { token, user }))
}
