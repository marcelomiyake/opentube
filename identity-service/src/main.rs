use std::{env, net::SocketAddr};

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use password_hash::rand_core::OsRng;
use platform_core::issue_token;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    jwt_secret: Vec<u8>,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    access_token: String,
    token_type: &'static str,
    expires_in: u64,
}

#[derive(sqlx::FromRow)]
struct UserRecord {
    id: Uuid,
    email: String,
    password_hash: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    let db_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let seed_email = env::var("SEED_EMAIL").unwrap_or_else(|_| "creator@opentube.local".into());
    let seed_password = env::var("SEED_PASSWORD")?;
    seed_creator(&pool, &seed_email, &seed_password).await?;

    let secret = env::var("JWT_SECRET")?;
    if secret.len() < 32 {
        return Err("JWT_SECRET must contain at least 32 bytes".into());
    }
    let state = AppState {
        db: pool,
        jwt_secret: secret.into_bytes(),
    };
    let app = app_router(state);

    let address: SocketAddr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8081".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "identity service listening");
    axum::serve(listener, app).await?;
    Ok(())
}

fn app_router(state: AppState) -> Router {
    Router::new()
        .route("/v1/login", post(login))
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

async fn seed_creator(
    pool: &PgPool,
    email: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let email = email.trim().to_lowercase();
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|error| std::io::Error::other(error.to_string()))?
        .to_string();
    sqlx::query("INSERT INTO identity.users (id, email, password_hash, display_name) VALUES ($1, $2, $3, $4) ON CONFLICT (email) DO NOTHING")
        .bind(Uuid::new_v4())
        .bind(email)
        .bind(hash)
        .bind("OpenTube Creator")
        .execute(pool)
        .await?;
    Ok(())
}

async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let email = request.email.trim().to_lowercase();
    if email.len() > 320 || request.password.len() > 1024 {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let user = sqlx::query_as::<_, UserRecord>(
        "SELECT id, email, password_hash FROM identity.users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let Some(user) = user else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let parsed =
        PasswordHash::new(&user.password_hash).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if Argon2::default()
        .verify_password(request.password.as_bytes(), &parsed)
        .is_err()
    {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let access_token = issue_token(user.id, &user.email, &state.jwt_secret, 60)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(LoginResponse {
        access_token,
        token_type: "Bearer",
        expires_in: 3600,
    }))
}

async fn live() -> StatusCode {
    StatusCode::OK
}

async fn ready(State(state): State<AppState>) -> StatusCode {
    if sqlx::query("SELECT 1").execute(&state.db).await.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;

    #[test]
    fn password_hash_verifies_only_the_original_password() {
        let salt = SaltString::generate(&mut OsRng);
        let encoded = Argon2::default()
            .hash_password(b"a local password", &salt)
            .unwrap()
            .to_string();
        let parsed = PasswordHash::new(&encoded).unwrap();
        assert!(
            Argon2::default()
                .verify_password(b"a local password", &parsed)
                .is_ok()
        );
        assert!(
            Argon2::default()
                .verify_password(b"wrong password", &parsed)
                .is_err()
        );
    }

    #[tokio::test]
    async fn seeded_creator_login_and_health_routes_work_against_test_postgres() {
        let Ok(database_url) = env::var("OPENTUBE_TEST_DATABASE_URL") else {
            return;
        };
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .expect("test Postgres is reachable");
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let email = format!("coverage-{}@example.test", Uuid::new_v4());
        let password = "coverage-test-password";
        seed_creator(&pool, &email, password).await.unwrap();
        let app = app_router(AppState {
            db: pool.clone(),
            jwt_secret: b"coverage-test-jwt-secret-at-least-32-bytes".to_vec(),
        });

        for path in ["/health/live", "/health/ready"] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK, "{path}");
        }

        let response = app
            .clone()
            .oneshot(json_request(
                "/v1/login",
                serde_json::json!({"email":email.to_uppercase(),"password":password}),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let token: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(token["token_type"], "Bearer");
        assert_eq!(token["expires_in"], 3600);
        assert!(
            platform_core::verify_token(
                token["access_token"].as_str().unwrap(),
                b"coverage-test-jwt-secret-at-least-32-bytes"
            )
            .is_ok()
        );

        for credentials in [
            serde_json::json!({"email":email,"password":"wrong"}),
            serde_json::json!({"email":"missing@example.test","password":"wrong"}),
            serde_json::json!({"email":"x".repeat(321),"password":"x"}),
            serde_json::json!({"email":"x@example.test","password":"x".repeat(1025)}),
        ] {
            let response = app
                .clone()
                .oneshot(json_request("/v1/login", credentials))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }

        sqlx::query("DELETE FROM identity.users WHERE email=$1")
            .bind(email)
            .execute(&pool)
            .await
            .unwrap();
    }

    fn json_request(uri: &str, value: serde_json::Value) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(value.to_string()))
            .unwrap()
    }
}
