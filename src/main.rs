use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use gloo::net::http::Request;




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
pub id: i32,
pub title: String,         
pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
pub id: i32,
pub username: String,
pub fullname: String,
pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
pub id: i32,
pub title: String,
pub summary: String,
}


const API_BASE: &str = "http://localhost:8080/api";


async fn fetch_courses() -> Result<Vec<Course>, String> {
let response = Request::get(&format!("{}/courses", API_BASE))
.send()
.await
.map_err(|e| e.to_string())?;

if response.ok() {
	response.json().await.map_err(|e| e.to_string())
} else {
	Err("Failed to load courses".to_string())
}
}

async fn fetch_articles() -> Result<Vec<Article>, String> {
let response = Request::get(&format!("{}/articles", API_BASE))
.send()
.await
.map_err(|e| e.to_string())?;

if response.ok() {
response.json().await.map_err(|e| e.to_string())
} else {
Err("Failed to load articles".to_string())
}
}

async fn login_request(username: String, password: String) -> Result<String, String> {
let response = Request::post(&format!("{}/auth/login", API_BASE))
.json(&serde_json::json!({
"username": username,
"password": password
}))
.map_err(|e| e.to_string())?
.send()
.await
.map_err(|e| e.to_string())?;

if response.ok() {
let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
let token = data["token"].as_str().unwrap_or("").to_string();
if token.is_empty() {
Err("No token received".to_string())
} else {
Ok(token)
}
} else {
Err("Invalid username or password".to_string())
}
}

async fn register_request(username: String, password: String, fullname: String, email: String) -> Result<String, String> {
let response = Request::post(&format!("{}/auth/register", API_BASE))
.json(&serde_json::json!({
"username": username,
"password": password,
"fullname": fullname,
"email": email
}))
.map_err(|e| e.to_string())?
.send()
.await
.map_err(|e| e.to_string())?;

if response.ok() {
let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
Ok(data["token"].as_str().unwrap_or("").to_string())
} else {
Err("Registration failed".to_string())
}
}

async fn enroll_course(course_id: i32, token: &str) -> Result<(), String> {
let response = Request::post(&format!("{}/courses/{}/enroll", API_BASE, course_id))
.header("Authorization", &format!("Bearer {}", token))
.send()
.await
.map_err(|e| e.to_string())?;

if response.ok() {
Ok(())
} else {
Err("Failed to enroll".to_string())
}
}

async fn fetch_my_courses(token: &str) -> Result<Vec<Course>, String> {
let response = Request::get(&format!("{}/my-courses", API_BASE))
.header("Authorization", &format!("Bearer {}", token))
.send()
.await
.map_err(|e| e.to_string())?;

if response.ok() {
response.json().await.map_err(|e| e.to_string())
} else {
Ok(vec![])
}
}


#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let (token, set_token) = create_signal(None::<String>);

    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(stored_token)) = storage.get_item("auth_token") {
                    set_token.set(Some(stored_token));
                }
            }
        }
    });

    create_effect(move |_| {
        let token_val = token.get();
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                match token_val {
                    Some(t) => {
                        let _ = storage.set_item("auth_token", &t);
                    }
                    None => {
                        let _ = storage.remove_item("auth_token");
                    }
                }
            }
        }
    });
    provide_context(token);
    provide_context(set_token);

let is_logged_in = move || token.get().is_some();

view! {
<Router>
<Title text="EduAcademy"/>
<nav class="navbar">
<div class="nav-container">
<a href="/" class="logo">"🎓 EduAcademy"</a>
<div class="nav-links">
<a href="/">"Home"</a>
<a href="/courses">"Courses"</a>
<a href="/articles">"Articles"</a>
<Show 
when=is_logged_in 
fallback=|| view! { <a href="/login" class="login-btn">"Login"</a> }
>
<a href="/my-courses">"My Courses"</a>
<button on:click=move |_| set_token.set(None) class="logout-btn">"Logout"</button>
</Show>
</div>
</div>
</nav>

<main>
<Routes>
<Route path="/" view=HomePage/>
<Route path="/courses" view=CoursesPage/>
<Route path="/articles" view=ArticlesPage/>
<Route path="/my-courses" view=MyCoursesPage/>
<Route path="/login" view=LoginPage/>
<Route path="/register" view=RegisterPage/>
</Routes>
</main>

<footer class="footer">
<p>"© 2024 EduAcademy. All rights reserved."</p>
</footer>
</Router>
}
}


#[component]
fn home_page() -> impl IntoView {
view! {
<div class="hero">
<h1>"Welcome to EduAcademy"</h1>
<p>"Learn the skills of tomorrow, today"</p>
<a href="/courses" class="cta-button">"Start Learning"</a>
</div>
<div class="features">
<div class="feature-card">
<div class="feature-icon">"🎓"</div>
<h3>"Expert Instructors"</h3>
<p>"Learn from the best"</p>
</div>
<div class="feature-card">
<div class="feature-icon">"📹"</div>
<h3>"HD Videos"</h3>
<p>"Watch anytime"</p>
</div>
<div class="feature-card">
<div class="feature-icon">"🎖️"</div>
<h3>"Certificates"</h3>
<p>"Get certified"</p>
</div>
</div>
}
}


#[component]
fn courses_page() -> impl IntoView {
let courses = create_resource(|| (), |_| fetch_courses());
let token = use_context::<ReadSignal<Option<String>>>().unwrap();

view! {
<div class="page-container">
<h2 class="page-title">"All Courses"</h2>
<Suspense fallback=|| view! { <div class="loading">"Loading courses..."</div> }>
{move || {
courses.get().map(|courses_res| {
match courses_res {
Ok(courses_list) => {
view! {
<div class="courses-grid">
{courses_list.into_iter().map(|course| {
let course_id = course.id;
let token_clone = token.get();
let enroll_action = create_action(move |_: &()| {
let id = course_id;
let t = token_clone.clone();
async move {
if let Some(tok) = t {
let _ = enroll_course(id, &tok).await;
}
}
});

view! {
<div class="course-card">
<h3>{course.title}</h3>
<p>{course.description}</p>
<Show 
when=move || token.get().is_some()
fallback=|| view! { <a href="/login" class="view-btn">"Login to Enroll"</a> }
>
<button on:click=move |_| enroll_action.dispatch(()) class="view-btn">
{move || if enroll_action.pending().get() { "Enrolling..." } else { "Enroll" }}
</button>
</Show>
</div>
}
}).collect::<Vec<_>>()}
</div>
}
}
Err(e) => view! { <div class="error-msg">{e.to_string()}</div> }
}
})
}}
</Suspense>
</div>
}
}

#[component]
fn articles_page() -> impl IntoView {
let articles = create_resource(|| (), |_| fetch_articles());

view! {
<div class="page-container">
<h2 class="page-title">"Articles"</h2>
<Suspense fallback=|| view! { <div class="loading">"Loading articles..."</div> }>
{move || {
articles.get().map(|articles_res| {
match articles_res {
Ok(articles_list) => {
view! {
<div class="courses-grid">
{articles_list.into_iter().map(|article| {
view! {
<div class="course-card">
<h3>{article.title}</h3>
<p>{article.summary}</p>
<a href="#" class="view-btn">"Read More"</a>
</div>
}
}).collect::<Vec<_>>()}
</div>
}
}
Err(e) => view! { <div class="error-msg">{e.to_string()}</div> }
}
})
}}
</Suspense>
</div>
}
}


#[component]
fn my_courses_page() -> impl IntoView {
let token = use_context::<ReadSignal<Option<String>>>().unwrap();
let my_courses = create_resource(move || token.get(), |t| async move {
if let Some(tok) = t {
fetch_my_courses(&tok).await
} else {
Ok(vec![])
}
});

view! {
<div class="page-container">
<h2 class="page-title">"My Courses"</h2>
<Suspense fallback=|| view! { <div class="loading">"Loading..."</div> }>
{move || {
my_courses.get().map(|courses| {
match courses {
Ok(courses_list) => {
if courses_list.is_empty() {
view! { <div class="loading">"You haven't enrolled in any courses yet."</div> }
} else {
view! {
<div class="courses-grid">
{courses_list.into_iter().map(|course| {
view! {
<div class="course-card">
<h3>{course.title}</h3>
<p>{course.description}</p>
<a href="/courses" class="view-btn">"Continue"</a>
</div>
}
}).collect::<Vec<_>>()}
</div>
}
}
}
Err(e) => view! { <div class="error-msg">{e.to_string()}</div> }
}
})
}}
</Suspense>
</div>
}
}


#[component]
fn login_page() -> impl IntoView {
let (username, set_username) = create_signal(String::new());
let (password, set_password) = create_signal(String::new());
let (error, set_error) = create_signal(None::<String>);

let set_token = use_context::<WriteSignal<Option<String>>>().unwrap();
let navigate = use_navigate();

let login_action = create_action(move |_: &()| {
let user = username.get();
let pass = password.get();
let set_token_clone = set_token.clone();
let navigate_clone = navigate.clone();

async move {
match login_request(user, pass).await {
Ok(token) => {
set_token_clone.set(Some(token));
navigate_clone("/", leptos_router::NavigateOptions::default());
}
Err(e) => {
set_error.set(Some(e));
}
}
}
});

view! {
<div class="auth-container">
<div class="auth-card">
<h2>"Login"</h2>
<form on:submit=move |ev| {
ev.prevent_default();
login_action.dispatch(());
}>
<input 
type="text" 
placeholder="Username"
prop:value=username
on:input=move |ev| set_username.set(event_target_value(&ev))
class="auth-input"
/>
<input 
type="password" 
placeholder="Password"
prop:value=password
on:input=move |ev| set_password.set(event_target_value(&ev))
class="auth-input"
/>
<button type="submit" class="auth-btn" disabled=login_action.pending()>
{move || if login_action.pending().get() { "Logging in..." } else { "Login" }}
</button>
</form>
{move || {
error.get().map(|err| view! { <div class="error-msg">{err}</div> })
}}
<p class="auth-footer">
"Don't have an account? " <a href="/register">"Sign up"</a>
</p>
</div>
</div>
}
}


#[component]
fn register_page() -> impl IntoView {
let (username, set_username) = create_signal(String::new());
let (password, set_password) = create_signal(String::new());
let (fullname, set_fullname) = create_signal(String::new());
let (email, set_email) = create_signal(String::new());
let (error, set_error) = create_signal(None::<String>);
let (success, set_success) = create_signal(None::<String>);

let set_token = use_context::<WriteSignal<Option<String>>>().unwrap();
let navigate = use_navigate();

let register_action = create_action(move |_: &()| {
let user = username.get();
let pass = password.get();
let name = fullname.get();
let mail = email.get();
let set_token_clone = set_token.clone();
let navigate_clone = navigate.clone();

async move {
match register_request(user, pass, name, mail).await {
Ok(token) => {
set_token_clone.set(Some(token));
navigate_clone("/", leptos_router::NavigateOptions::default());
set_success.set(Some("Registration successful!".to_string()));
}
Err(e) => {
set_error.set(Some(e));
}
}
}
});

view! {
<div class="auth-container">
<div class="auth-card">
<h2>"Sign Up"</h2>
<form on:submit=move |ev| {
ev.prevent_default();
register_action.dispatch(());
}>
<input 
type="text" 
placeholder="Full Name"
prop:value=fullname
on:input=move |ev| set_fullname.set(event_target_value(&ev))
class="auth-input"
/>
<input 
type="email" 
placeholder="Email"
prop:value=email
on:input=move |ev| set_email.set(event_target_value(&ev))
class="auth-input"
/>
<input 
type="text" 
placeholder="Username"
prop:value=username
on:input=move |ev| set_username.set(event_target_value(&ev))
class="auth-input"
/>
<input 
type="password" 
placeholder="Password"
prop:value=password
on:input=move |ev| set_password.set(event_target_value(&ev))
class="auth-input"
/>
<button type="submit" class="auth-btn" disabled=register_action.pending()>
{move || if register_action.pending().get() { "Creating account..." } else { "Sign Up" }}
</button>
</form>
{move || {
error.get().map(|err| view! { <div class="error-msg">{err}</div> })
}}
{move || {
success.get().map(|msg| view! { <div class="success-msg">{msg}</div> })
}}
<p class="auth-footer">
"Already have an account? " <a href="/login">"Login"</a>
</p>
</div>
</div>
}
}


fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
