# Utoipa - Auto-generated OpenAPI Documentation

Utoipa is a Rust library that automatically generates OpenAPI 3.1 documentation from your code using procedural macros. It takes a code-first approach, allowing developers to annotate their Rust types and API handlers to produce complete OpenAPI specifications without manual YAML or JSON editing. The library is framework-agnostic but provides enhanced integrations for popular web frameworks like actix-web, axum, and rocket.

The core functionality centers around derive macros (`ToSchema`, `OpenApi`, `IntoParams`) and attribute macros (`#[utoipa::path]`) that extract type information and API metadata directly from Rust code. It automatically collects schemas recursively, recognizes request and response bodies, supports generic types, and provides built-in support for common Rust types and popular crates like chrono, uuid, and serde. The library also includes companion crates for serving documentation through Swagger UI, RapiDoc, ReDoc, and Scalar interfaces.

## API Documentation

### Derive OpenAPI Schema from Rust Structs

Generate OpenAPI schema definitions from Rust types using the `ToSchema` derive macro.

```rust
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, ToSchema)]
struct Pet {
    /// Unique identifier for the pet
    #[schema(example = 1)]
    id: u64,
    /// Pet's name
    #[schema(example = "Fluffy")]
    name: String,
    /// Optional age in years
    age: Option<i32>,
    /// Whether the pet is available for adoption
    #[serde(default)]
    available: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
enum PetStatus {
    Available,
    Pending,
    Adopted,
}
```

### Document API Endpoints with Path Macro

Annotate handler functions to automatically generate OpenAPI path documentation with parameters, request bodies, and responses.

```rust
use utoipa::path;
use actix_web::{get, post, web::{Path, Json}, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
struct Pet {
    id: u64,
    name: String,
    age: Option<i32>,
}

#[derive(Serialize, Deserialize, ToSchema)]
enum ErrorResponse {
    NotFound(String),
    Conflict(String),
}

/// Get pet by ID
///
/// Retrieves a pet from the database by its unique identifier
#[utoipa::path(
    get,
    path = "/pets/{id}",
    tag = "pets",
    responses(
        (status = 200, description = "Pet found successfully", body = Pet),
        (status = 404, description = "Pet not found", body = ErrorResponse)
    ),
    params(
        ("id" = u64, Path, description = "Pet database ID")
    )
)]
#[get("/pets/{id}")]
async fn get_pet_by_id(id: Path<u64>) -> impl Responder {
    HttpResponse::Ok().json(Pet {
        id: id.into_inner(),
        name: "Fluffy".to_string(),
        age: Some(3),
    })
}

/// Create new pet
///
/// Adds a new pet to the database
#[utoipa::path(
    post,
    path = "/pets",
    tag = "pets",
    responses(
        (status = 201, description = "Pet created successfully", body = Pet),
        (status = 409, description = "Pet already exists", body = ErrorResponse)
    )
)]
#[post("/pets")]
async fn create_pet(pet: Json<Pet>) -> impl Responder {
    HttpResponse::Created().json(pet.into_inner())
}
```

### Generate Complete OpenAPI Specification

Create the OpenAPI document by deriving `OpenApi` and registering paths and schemas.

```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Pet Store API",
        version = "1.0.0",
        description = "A simple pet store API",
        contact(
            name = "API Support",
            email = "support@petstore.com"
        )
    ),
    paths(
        get_pet_by_id,
        create_pet,
        list_pets,
        update_pet,
        delete_pet
    ),
    components(schemas(Pet, ErrorResponse, PetStatus)),
    tags(
        (name = "pets", description = "Pet management endpoints")
    )
)]
struct ApiDoc;

fn main() {
    // Generate OpenAPI JSON
    let openapi_json = ApiDoc::openapi().to_pretty_json().unwrap();
    println!("{}", openapi_json);

    // Or get the OpenAPI object directly for modification
    let mut openapi = ApiDoc::openapi();
    openapi.info.title = "Modified Pet Store API".to_string();
}
```

### Query Parameters with IntoParams

Define query parameter structures that automatically generate OpenAPI parameter documentation.

```rust
use utoipa::IntoParams;
use serde::Deserialize;
use actix_web::{get, web::Query, HttpResponse, Responder};

#[derive(Deserialize, IntoParams)]
struct SearchQuery {
    /// Search term for pet name (case insensitive)
    #[param(example = "fluffy")]
    name: Option<String>,
    /// Filter by minimum age
    #[param(minimum = 0)]
    min_age: Option<i32>,
    /// Filter by availability status
    available: Option<bool>,
    /// Page number for pagination
    #[param(default = 1, minimum = 1)]
    page: i32,
    /// Items per page
    #[param(default = 20, minimum = 1, maximum = 100)]
    limit: i32,
}

/// Search pets
///
/// Search for pets using various filter criteria
#[utoipa::path(
    get,
    path = "/pets/search",
    tag = "pets",
    params(SearchQuery),
    responses(
        (status = 200, description = "List of matching pets", body = Vec<Pet>)
    )
)]
#[get("/pets/search")]
async fn search_pets(query: Query<SearchQuery>) -> impl Responder {
    let filtered_pets = vec![]; // Perform search logic
    HttpResponse::Ok().json(filtered_pets)
}
```

### Add Security Schemes

Configure API authentication and security requirements using the `Modify` trait.

```rust
use utoipa::{Modify, OpenApi};
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme, Http, HttpAuthScheme};

#[derive(OpenApi)]
#[openapi(
    paths(get_pet_by_id, delete_pet),
    components(schemas(Pet)),
    modifiers(&SecurityAddon),
    tags((name = "pets", description = "Pet management"))
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            // Add API key authentication
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key")))
            );

            // Add bearer token authentication
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer))
            );
        }
    }
}

/// Delete pet by ID (requires authentication)
#[utoipa::path(
    delete,
    path = "/pets/{id}",
    tag = "pets",
    responses(
        (status = 200, description = "Pet deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Pet not found")
    ),
    params(
        ("id" = u64, Path, description = "Pet ID to delete")
    ),
    security(
        ("api_key" = [])
    )
)]
async fn delete_pet(id: Path<u64>) -> impl Responder {
    HttpResponse::Ok().finish()
}
```

### Serve Documentation with Swagger UI

Integrate Swagger UI to serve interactive API documentation alongside your API.

```rust
use actix_web::{App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(paths(get_pet_by_id, create_pet), components(schemas(Pet)))]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // Register API routes
            .service(get_pet_by_id)
            .service(create_pet)
            // Serve Swagger UI at /swagger-ui/
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

### Axum Integration with OpenApiRouter

Use utoipa-axum for seamless integration with axum framework, automatically collecting routes and generating OpenAPI specs.

```rust
use axum::{Json, extract::{Path, State}, response::IntoResponse, http::StatusCode};
use std::sync::Arc;
use tokio::sync::Mutex;
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone, serde::Serialize, ToSchema)]
struct Pet {
    id: u64,
    name: String,
}

type Store = Arc<Mutex<Vec<Pet>>>;

/// List all pets
#[utoipa::path(
    get,
    path = "/pets",
    tag = "pets",
    responses(
        (status = 200, description = "List of all pets", body = Vec<Pet>)
    )
)]
async fn list_pets(State(store): State<Store>) -> Json<Vec<Pet>> {
    let pets = store.lock().await.clone();
    Json(pets)
}

/// Get pet by ID
#[utoipa::path(
    get,
    path = "/pets/{id}",
    tag = "pets",
    responses(
        (status = 200, description = "Pet found", body = Pet),
        (status = 404, description = "Pet not found")
    ),
    params(
        ("id" = u64, Path, description = "Pet ID")
    )
)]
async fn get_pet(
    Path(id): Path<u64>,
    State(store): State<Store>
) -> Result<Json<Pet>, StatusCode> {
    store.lock().await
        .iter()
        .find(|pet| pet.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

#[derive(OpenApi)]
#[openapi(tags((name = "pets", description = "Pet management API")))]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let store = Arc::new(Mutex::new(vec![]));

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(list_pets))
        .routes(routes!(get_pet))
        .with_state(store)
        .split_for_parts();

    let app = router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}
```

### Runtime OpenAPI Modification

Modify the generated OpenAPI specification at runtime using direct field access or builder patterns.

```rust
use utoipa::{OpenApi, openapi::{OpenApiBuilder, Info, Server, ServerBuilder}};

#[derive(OpenApi)]
#[openapi(paths(get_pet_by_id), components(schemas(Pet)))]
struct ApiDoc;

fn main() {
    // Direct modification
    let mut openapi = ApiDoc::openapi();
    openapi.info.title = "Custom Pet API".to_string();
    openapi.info.version = "2.0.0".to_string();
    openapi.servers = Some(vec![
        Server::new("https://api.production.com"),
        Server::new("https://api.staging.com"),
    ]);

    // Using builder pattern
    let openapi_modified: OpenApiBuilder = ApiDoc::openapi().into();
    let openapi = openapi_modified
        .info(Info::new("Pet API", "2.0.0"))
        .servers(Some(vec![
            ServerBuilder::new()
                .url("https://api.example.com")
                .description(Some("Production server"))
                .build(),
        ]))
        .build();

    // Export to JSON
    let json = openapi.to_pretty_json().unwrap();
    std::fs::write("openapi.json", json).unwrap();
}
```

### Custom Response Types

Define reusable response types and error responses with the `ToResponse` and `IntoResponses` derive macros.

```rust
use utoipa::{ToSchema, ToResponse, IntoResponses};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, ToSchema, ToResponse)]
#[response(description = "Successful pet response")]
struct PetResponse {
    #[schema(example = 1)]
    id: u64,
    #[schema(example = "Fluffy")]
    name: String,
}

#[derive(Serialize, Deserialize, ToSchema, IntoResponses)]
enum ApiError {
    #[response(status = 404, description = "Resource not found")]
    NotFound { message: String },

    #[response(status = 400, description = "Invalid request")]
    BadRequest {
        #[schema(example = "Invalid pet ID format")]
        message: String
    },

    #[response(status = 500, description = "Internal server error")]
    InternalError { message: String },
}

/// Get pet with custom response types
#[utoipa::path(
    get,
    path = "/pets/{id}",
    responses(
        (status = 200, response = PetResponse),
        ApiError
    ),
    params(
        ("id" = u64, Path, description = "Pet ID")
    )
)]
async fn get_pet_custom(id: Path<u64>) -> Result<PetResponse, ApiError> {
    if id.into_inner() == 0 {
        return Err(ApiError::BadRequest {
            message: "Pet ID must be greater than 0".to_string()
        });
    }

    Ok(PetResponse {
        id: 1,
        name: "Fluffy".to_string(),
    })
}
```

## Summary

Utoipa streamlines API documentation by eliminating the need for separate documentation files and keeping API specs synchronized with implementation code. The primary use cases include building RESTful APIs with automatic OpenAPI generation, creating type-safe API documentation that updates as code changes, and integrating interactive API explorers (Swagger UI, RapiDoc, ReDoc, Scalar) directly into web applications. It's particularly valuable for teams maintaining large APIs where manual documentation quickly becomes outdated.

The library integrates with popular Rust web frameworks through dedicated companion crates: `utoipa-actix-web` for actix-web applications, `utoipa-axum` for axum services, and enhanced support for rocket. It can also be used framework-independently by manually constructing OpenAPI specifications using Rust types. The integration pattern typically involves annotating types with derive macros, marking handlers with path attributes, deriving an OpenApi struct that registers all components, and optionally serving the specification through UI crates. This approach provides compile-time guarantees about API structure while maintaining the flexibility to modify generated documentation at runtime.

