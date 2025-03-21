use std::collections::HashMap;

use axum::{body::{Body, Bytes}, extract::{rejection::JsonRejection, Multipart, Path, Query, Request}, response::Response, routing::{get, post}, serve, Form, Json, Router};
use axum_extra::{body, extract::{cookie::{self, Cookie}, CookieJar}, response};
use axum_test::{multipart::{MultipartForm, Part}, TestServer};
use http::{header, request, HeaderMap, HeaderValue, Method, StatusCode, Uri};
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