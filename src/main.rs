use axum::{extract::Request, routing::{get, post}, serve, Router};
use axum_extra::response;
use axum_test::TestServer;
use http::{Method, Uri};
use tokio::net::TcpListener;

// Setup
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async {"Hello, World!"}));
    
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    // menjalankan server
    serve(listener, app).await.unwrap();
}


// Axum Test
#[tokio::test]
async fn test_axum() {
    let app = Router::new()
        .route("/", get(|| async {"Hello, World!"}));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/").await;

    response.assert_status_ok();
    response.assert_text("Hello, World!");
}


// Router atau Routing
#[tokio::test]
async fn test_method_routing() {
    async fn hello_world() -> String {
        "Hello, World!".to_string()
    }

    let app = Router::new()
        .route("/get", get(hello_world))
        .route("/post", post(hello_world));

    let server = TestServer::new(app).unwrap();

    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Hello, World!");

    let response = server.post("/post").await;
    response.assert_status_ok();
    response.assert_text("Hello, World!");
}


// Request
#[tokio::test]
async fn test_request() {
    async fn hello_world(request: Request) -> String {
        format!("Hello {}", request.method())
    }

    let app = Router::new()
        .route("/get", get(hello_world))
        .route("/post", post(hello_world));

    let server = TestServer::new(app).unwrap();

    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");

    let response = server.post("/post").await;
    response.assert_status_ok();
    response.assert_text("Hello POST");
}


// Extractor
#[tokio::test]
async fn test_uri() {
    async fn hello_world(uri: Uri, method: Method) -> String {
        format!("Hello {} {}", method, uri.path())
    }

    let app = Router::new()
        .route("/get", get(hello_world))
        .route("/post", post(hello_world));

    let server = TestServer::new(app).unwrap();

    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Hello GET /get");

    let response = server.post("/post").await;
    response.assert_status_ok();
    response.assert_text("Hello POST /post");

}
