use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use axum::{body::{Body, Bytes}, error_handling::HandleError, extract::{rejection::JsonRejection, Multipart, Path, Query, Request, State}, middleware::{from_fn, map_request, Next}, response::{IntoResponse, Response}, routing::{get, post}, serve, Extension, Form, Json, Router};
use axum_extra::{body, extract::{cookie::{self, Cookie}, CookieJar}, response};
use axum_test::{multipart::{MultipartForm, Part}, TestServer};
use http::{header, method, request, HeaderMap, HeaderValue, Method, StatusCode, Uri};
use serde::{Deserialize, Serialize};
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


// Common Extractor
    // Query Parameter
#[tokio::test]
async fn test_query() {
    async fn hello_world(Query(params) : Query<HashMap<String, String>>) -> String {
        let name = params.get("name").unwrap();
        format!("Hello {}", name)
    }

    let app = Router::new()
        .route("/get", get(hello_world));

    let server = TestServer::new(app).unwrap();

    let response = server.get("/get").add_query_param("name", "Aqil").await;
    response.assert_status_ok();
    response.assert_text("Hello Aqil");

}


// Common Extractor
    // Header Extractor
    #[tokio::test]
    async fn test_header() {
        async fn hello_world(headers: HeaderMap) -> String {
            let name = headers["name"].to_str().unwrap();
            format!("Hello {}", name)
        }
    
        let app = Router::new()
            .route("/get", get(hello_world));
    
        let server = TestServer::new(app).unwrap();
    
        let response = server.get("/get").add_header("name", "Aqil").await;
        response.assert_status_ok();
        response.assert_text("Hello Aqil");
    
    }


// Path Paraneter Extractor
#[tokio::test]
async fn test_path_parameter() {
    async fn hello_world(Path((id, id_category)) : Path<(String, String)>) -> String {
        format!("Product {}, Category {}", id, id_category)
    }
    
    let app = Router::new()
        .route("/products/{id}/categories/{id_category}", get(hello_world));
    
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/products/1/categories/3").await;
    response.assert_status_ok();
    response.assert_text("Product 1, Category 3");
    
}


// Body Extractor
#[tokio::test]
async fn test_body_string() {
    async fn hello_world(body: String) -> String {
        format!("Body {}", body)
    }
    
    let app = Router::new()
        .route("/post", post(hello_world));
    
    let server = TestServer::new(app).unwrap();
    
    let response = server.post("/post").text("This is body").await;
    response.assert_status_ok();
    response.assert_text("Body This is body");
    
}

// Json body extractor
#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[tokio::test]
async fn test_json_body() {
    async fn hello_world(Json(request) : Json<LoginRequest>) -> String {
        format!("Hello {}", request.username)
    }
    
    let app = Router::new()
        .route("/post", post(hello_world));

    let request = LoginRequest {
        username: "Aqil".to_string(),
        password: "12345".to_string(),
    };
    
    let server = TestServer::new(app).unwrap();
    
    let response = server.post("/post").json(&request).await;
    response.assert_status_ok();
    response.assert_text("Hello Aqil");
    
}


// Json Error
#[tokio::test]
async fn test_json_error() {
    async fn hello_world(payload: Result<Json<LoginRequest>, JsonRejection>) -> String {
        match payload {
            Ok(request) => {
                format!("Hello {}", request.username)
            }
            Err(error) => {
                format!("Error {:?}", error)
            }
        }
    }
    
    let app = Router::new()
        .route("/post", post(hello_world));

    let request = LoginRequest {
        username: "Aqil".to_string(),
        password: "<PASSWORD>".to_string(),
    };
    
    let server = TestServer::new(app).unwrap();
    
    // menggunakan json yang valid
    let response = server.post("/post").json(&request).await;
    response.assert_status_ok();
    response.assert_text("Hello Aqil");

    // menngunakan json yang tidak valid
    let response = server.post("/post").text("tidak valid").await;
    response.assert_status_ok();
    response.assert_text("Error MissingJsonContentType(MissingJsonContentType");
    
}


 // Response
 #[tokio::test]
async fn test_response() {
    async fn hello_world(request: Request) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("X-Owner", "Aqil")
            .body(Body::from(format!("Hello {}", request.method())))
            .unwrap()
    }
    
    let app = Router::new()
        .route("/get", get(hello_world));
    
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");
    response.assert_header("X-Owner", "Aqil");  
}


// json response
#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    token: String,
}

#[tokio::test]
async fn test_response_json() {
    async fn hello_world() -> Json<LoginResponse> {
        Json(LoginResponse { 
            token: "token".to_string() 
        })
    }
    
    let app = Router::new()
        .route("/get", get(hello_world));
    
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("{\"token\":\"token\"}"); 
}


// tiple into response
#[tokio::test]
async fn test_response_tuple() {
    async fn hello_world() -> (Response<()>, Json<LoginResponse>) {
        (
            Response::builder()
                .status(StatusCode::OK)
                .header("X-Owner", "Aqil")
                .body(())
                .unwrap(),
            Json(LoginResponse {
                token: "token".to_string(),
            }),
        )
    }
    
    let app = Router::new()
        .route("/get", get(hello_world));
    
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("{\"token\":\"token\"}"); 
    response.assert_header("X-Owner", "Aqil"); 
}

#[tokio::test]
async fn test_response_tuple3() {
    async fn hello_world() -> (StatusCode, HeaderMap, Json<LoginResponse>) {
        let mut header = HeaderMap::new();
        header.insert("X-Owner", HeaderValue::from_str("Aqil").unwrap());

        (
            StatusCode::OK,
            header,
            Json(LoginResponse {
                token: "token".to_string(),
            }),
        )
    }
    
    let app = Router::new()
        .route("/get", get(hello_world));
    
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("{\"token\":\"token\"}"); 
    response.assert_header("X-Owner", "Aqil"); 
}


// Form Request
#[tokio::test]
async fn test_form() {
    async fn hello_world(Form(request) : Form<LoginRequest>) -> String {
        format!("Hello {}", request.username)
    }
    
    let app = Router::new()
        .route("/post", post(hello_world));

    let request = LoginRequest {
        username: "Aqil".to_string(),
        password: "12345".to_string(),
    };
    
    let server = TestServer::new(app).unwrap();
    
    let response = server.post("/post").form(&request).await;
    response.assert_status_ok();
    response.assert_text("Hello Aqil");
    
}


// Multiple Request
#[tokio::test]
async fn test_multipart() {
    async fn hello_world(mut payload: Multipart) -> String {
        let mut profile: Bytes = Bytes::new();
        let mut username = "".to_string();

        while let Some (field) = payload.next_field().await.unwrap() {
            if field.name().unwrap_or("") == "profile" {
                profile = field.bytes().await.unwrap()
            } else if field.name().unwrap_or("") == "username" {
                username = field.text().await.unwrap()
            }
        }

        assert!(profile.len() > 0);
        format!("Hello {}", username)
    }
    
    let app = Router::new()
        .route("/post", post(hello_world));

    let request = MultipartForm::new()
        .add_text("username", "Aqil")
        .add_text("password", "rahasia")
        .add_part("profile", Part::bytes(Bytes::from("Contoh")));
    
    let server = TestServer::new(app).unwrap();
    
    let response = server.post("/post").multipart(request).await;
    response.assert_status_ok();
    response.assert_text("Hello Aqil");
    
}


// Cookie
#[tokio::test]
async fn test_cookie_response() {
    async fn hello_world(query: Query<HashMap<String, String>>) -> (CookieJar, String) {
        let name = query.get("name").unwrap();

        (
            CookieJar::new().add(Cookie::new("name", name.clone())),
            format!("Hello {}", name.clone()),
        )
    }
    
    let app = Router::new()
        .route("/get", get(hello_world));
    
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/get").add_query_param("name", "Aqil").await;
    response.assert_status_ok();
    response.assert_text("Hello Aqil");
    response.assert_header("Set-Cookie", "name=Aqil");
    
}


// cookie request
#[tokio::test]
async fn test_cookie_request() {
    async fn hello_world(cookie: CookieJar) -> String {
        let name = cookie.get("name").unwrap().value();

        format!("Hello {}", name)
    }
    
    let app = Router::new()
        .route("/get", get(hello_world));
    
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/get").add_header("Cookie", "name=Aqil").await;
    response.assert_status_ok();
    response.assert_text("Hello Aqil");
    
}


// Middleware
async fn log_middleware(request: Request, next: Next) -> Response {
    println!("Recieve request {} {}", request.method(), request.uri());
    let response = next.run(request).await;
    println!("Sen response {}", response.status());
    response
}

async fn request_id_middleware<T>(mut request: Request<T>) -> Request<T> {
    let request_id = "12345";
    request
        .headers_mut()
        .insert("X-Request-Id", request_id.parse().unwrap());
    request
}

#[tokio::test]
async fn test_middleware() {
    async fn hello_world(method: Method, header_map: HeaderMap) -> String {
        println!("Execute handler");
        let request_id = header_map.get("X-Request-Id").unwrap().to_str().unwrap();

        format!("Hello {} {}", method, request_id)
    }
    
    let app = Router::new()
        .route("/get", get(hello_world))
        .layer(map_request(request_id_middleware))
        .layer(from_fn(log_middleware));
    
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/get").add_header("Cookie", "name=Aqil").await;
    response.assert_status_ok();
    response.assert_text("Hello GET 12345");
    
}


// Error Handling
struct AppError {
    code: i32,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::from_u16(self.code as u16).unwrap(),
            self.message,
        )

            .into_response()
    }
}

#[tokio::test]
async fn test_error_handling() {
    async fn hello_world(method: Method) -> Result<String, AppError> {
        if method == Method::POST {
            Ok("OK".to_string())
        } else {
            Err(AppError { 
                code: 400, 
                message: "Bad Request".to_string(), 
            })
        }
    }
    
    let app = Router::new()
        .route("/get", get(hello_world))
        .route("/post", post(hello_world));
    
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/get").await;
    response.assert_status(StatusCode::BAD_REQUEST);
    response.assert_text("Bad Request");
    
}

// unexpected error
#[tokio::test]
async fn test_unexpected_eeror() {
    async fn route(request: Request) -> Result<Response, anyhow::Error> {
        if request.method() == Method::POST {
            Ok(Response::builder().status(StatusCode::OK).body(Body::from("OK"))?)
        } else {
            //Err(anyhow::Error::msg("Bad Request"))
            Err(anyhow!("Bad Request"))
        }
    }

    async fn handle_error(err: anyhow::Error) -> (StatusCode, String) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal Server Error : {}", err),
        )
    }

    let route_service = tower::service_fn(route);

    let app = Router::new()
        .route_service("/get", HandleError::new(route_service, handle_error));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/get").await;
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    response.assert_text("Internal Server Error : Bad Request");

}


// State
// state extractor
struct DatabaseConfig {
    total: i32
}

#[tokio::test]
async fn test_state_extractor() {
    let database_state = Arc::new(DatabaseConfig{total: 100});

    async fn hello_world(State(database) : State<Arc<DatabaseConfig>>) -> String {
        
        format!("Total {}", database.total)
    }
    
    let app = Router::new()
        .route("/get", get(hello_world))
        .with_state(database_state);

    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Total 100");
    
}

// state extension
#[tokio::test]
async fn test_state_extension() {
    let database_state = Arc::new(DatabaseConfig{total: 100});

    async fn hello_world(Extension(database) : Extension<Arc<DatabaseConfig>>) -> String {
        
        format!("Total {}", database.total)
    }
    
    let app = Router::new()
        .route("/get", get(hello_world))
        .layer(Extension(database_state));

    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Total 100");
    
}

// closure capture
#[tokio::test]
async fn test_state_closure_capture() {
    let database_state = Arc::new(DatabaseConfig{total: 100});

    async fn hello_world(database : Arc<DatabaseConfig>) -> String {
        
        format!("Total {}", database.total)
    }
    
    let app = Router::new()
        .route("/get", get({
            let database_state = Arc::clone(&database_state);
            move || hello_world(database_state)
        }))
        .layer(Extension(database_state));

    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/get").await;
    response.assert_status_ok();
    response.assert_text("Total 100");
    
}


// Multiple Router
#[tokio::test]
async fn test_multiple_route() {
    async fn hello_world(method: Method) -> String {
        
        format!("Hello {}", method)
    }

    let first = Router::new().route("/first", get(hello_world));
    let second = Router::new().route("/second", get(hello_world));
    
    let app = Router::new()
        .merge(first)
        .merge(second);

    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/first").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");

    let response = server.get("/second").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");
    
}

// nest multiple router
#[tokio::test]
async fn test_multiple_route_nest() {
    async fn hello_world(method: Method) -> String {
        
        format!("Hello {}", method)
    }

    let first = Router::new().route("/first", get(hello_world));
    let second = Router::new().route("/second", get(hello_world));
    
    let app = Router::new()
        .nest("/api/users", first)
        .nest("/api/products", second);

    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/api/users/first").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");

    let response = server.get("/api/products/second").await;
    response.assert_status_ok();
    response.assert_text("Hello GET");
    
}