use crate::cache::Cache;
use crate::config::{Config, ServerConfig};
use crate::container::App;
use crate::http::{HttpResponse, Request};
use crate::middleware::{Middleware, MiddlewareChain, MiddlewareRegistry};
use crate::routing::Router;
use crate::websocket::handle_ws_upgrade;
use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Pre-routing WebSocket interceptor.
///
/// Called for every WS upgrade request before Ferro routing.
/// Returns `Ok(Response)` to handle the request, `Err(Request)` to decline
/// and pass to normal routing (including the built-in `/_ferro/ws` check).
type WsInterceptor = Box<
    dyn Fn(
            hyper::Request<hyper::body::Incoming>,
        ) -> Result<hyper::Response<Full<Bytes>>, hyper::Request<hyper::body::Incoming>>
        + Send
        + Sync,
>;

/// HTTP server that binds routes, middleware, and optional WebSocket handling.
pub struct Server {
    router: Arc<Router>,
    middleware: MiddlewareRegistry,
    host: String,
    port: u16,
    ws_interceptor: Option<Arc<WsInterceptor>>,
}

impl Server {
    /// Create a server with default host/port and no global middleware.
    pub fn new(router: impl Into<Router>) -> Self {
        Self {
            router: Arc::new(router.into()),
            middleware: MiddlewareRegistry::new(),
            host: "127.0.0.1".to_string(),
            port: 8080,
            ws_interceptor: None,
        }
    }

    /// Create a server from environment configuration, booting all services.
    pub fn from_config(router: impl Into<Router>) -> Self {
        // Initialize the App container
        App::init();

        // Boot all auto-registered services from #[service(ConcreteType)]
        App::boot_services();

        let config = Config::get::<ServerConfig>().unwrap_or_else(ServerConfig::from_env);
        Self {
            router: Arc::new(router.into()),
            // Pull global middleware registered via global_middleware! in bootstrap.rs
            middleware: MiddlewareRegistry::from_global(),
            host: config.host,
            port: config.port,
            ws_interceptor: None,
        }
    }

    /// Set a WebSocket interceptor that runs before all routing.
    ///
    /// The interceptor receives every WS upgrade request first.
    /// Return `Ok(response)` to handle the connection; return `Err(request)` to
    /// decline and let normal routing (including `/_ferro/ws`) proceed.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Server::from_config(router)
    ///     .ws_interceptor(|req| {
    ///         if req.uri().path().starts_with("/sessions/") {
    ///             Ok(my_ws_handler(req))
    ///         } else {
    ///             Err(req) // pass to /_ferro/ws
    ///         }
    ///     })
    ///     .run()
    ///     .await;
    /// ```
    pub fn ws_interceptor<F>(mut self, handler: F) -> Self
    where
        F: Fn(
                hyper::Request<hyper::body::Incoming>,
            )
                -> Result<hyper::Response<Full<Bytes>>, hyper::Request<hyper::body::Incoming>>
            + Send
            + Sync
            + 'static,
    {
        self.ws_interceptor = Some(Arc::new(Box::new(handler)));
        self
    }

    /// Add global middleware (runs on every request)
    ///
    /// For route-specific middleware, use `.middleware(M)` on the route itself.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Server::from_config(router)
    ///     .middleware(LoggingMiddleware)  // Global
    ///     .middleware(CorsMiddleware)     // Global
    ///     .run()
    ///     .await;
    /// ```
    pub fn middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middleware = self.middleware.append(middleware);
        self
    }

    /// Override the listen host address.
    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    /// Override the listen port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    fn get_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host.parse().unwrap(), self.port)
    }

    /// Start listening and serving requests until the process is terminated.
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Bootstrap cache (Redis with in-memory fallback)
        Cache::bootstrap().await;

        let addr: SocketAddr = self.get_addr();
        let listener = TcpListener::bind(addr).await?;

        println!("Ferro server running on http://{addr}");

        let router = self.router;
        let middleware = Arc::new(self.middleware);
        let ws_interceptor = self.ws_interceptor;

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let router = router.clone();
            let middleware = middleware.clone();
            let ws_interceptor = ws_interceptor.clone();

            tokio::spawn(async move {
                let service = service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                    let router = router.clone();
                    let middleware = middleware.clone();
                    let ws_interceptor = ws_interceptor.clone();
                    async move {
                        Ok::<_, Infallible>(
                            handle_request(router, middleware, ws_interceptor, req).await,
                        )
                    }
                });

                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service)
                    .with_upgrades()
                    .await
                {
                    eprintln!("Error serving connection: {err:?}");
                }
            });
        }
    }
}

async fn handle_request(
    router: Arc<Router>,
    middleware_registry: Arc<MiddlewareRegistry>,
    ws_interceptor: Option<Arc<WsInterceptor>>,
    mut req: hyper::Request<hyper::body::Incoming>,
) -> hyper::Response<Full<Bytes>> {
    // Application WS interceptor runs before /_ferro/ws
    if let Some(ref interceptor) = ws_interceptor {
        if hyper_tungstenite::is_upgrade_request(&req) {
            match interceptor(req) {
                Ok(response) => return response,
                Err(returned_req) => {
                    // Interceptor declined — continue with returned request
                    req = returned_req;
                }
            }
        }
    }

    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("");

    // WebSocket upgrade at /_ferro/ws (must run before middleware/routing)
    if path == "/_ferro/ws" && hyper_tungstenite::is_upgrade_request(&req) {
        return handle_ws_upgrade(req);
    }

    // Built-in framework endpoints at /_ferro/*
    // Uses framework prefix to avoid conflicts with user-defined routes
    if path.starts_with("/_ferro/") && method == hyper::Method::GET {
        return match path.as_str() {
            "/_ferro/health" => health_response(query).await,
            "/_ferro/routes" => crate::debug::handle_routes(),
            "/_ferro/middleware" => crate::debug::handle_middleware(),
            "/_ferro/services" => crate::debug::handle_services(),
            "/_ferro/metrics" => crate::debug::handle_metrics(),
            "/_ferro/queue/jobs" => crate::debug::handle_queue_jobs().await,
            "/_ferro/queue/stats" => crate::debug::handle_queue_stats().await,
            "/_ferro/ferro-base.css" => serve_ferro_base_css(),
            _ => HttpResponse::text("404 Not Found").status(404).into_hyper(),
        };
    }

    // Note: Inertia context is now read directly from Request headers
    // via req.is_inertia(), req.inertia_version(), etc.
    // No thread-local storage needed - this is async-safe.

    let response = match router.match_route(&method, &path) {
        Some((handler, params, route_pattern)) => {
            let request = Request::new(req)
                .with_params(params)
                .with_route_pattern(route_pattern.clone());

            // Build middleware chain
            let mut chain = MiddlewareChain::new();

            // 1. Add global middleware
            chain.extend(middleware_registry.global_middleware().iter().cloned());

            // 2. Add route-level middleware (already boxed)
            let route_middleware = router.get_route_middleware(&route_pattern);
            chain.extend(route_middleware);

            // 3. Execute chain with handler
            let response = chain.execute(request, handler).await;

            // Unwrap the Result - both Ok and Err contain HttpResponse
            let http_response = response.unwrap_or_else(|e| e);
            http_response.into_hyper()
        }
        None => {
            // Try static file serving before fallback (only GET/HEAD)
            if method == hyper::Method::GET || method == hyper::Method::HEAD {
                if let Some(response) = crate::static_files::try_serve_static_file(&path).await {
                    return response;
                }
            }

            // Check for fallback handler
            if let Some((fallback_handler, fallback_middleware)) = router.get_fallback() {
                let request = Request::new(req).with_params(std::collections::HashMap::new());

                // Build middleware chain for fallback
                let mut chain = MiddlewareChain::new();

                // 1. Add global middleware
                chain.extend(middleware_registry.global_middleware().iter().cloned());

                // 2. Add fallback-specific middleware
                chain.extend(fallback_middleware);

                // 3. Execute chain with fallback handler
                let response = chain.execute(request, fallback_handler).await;

                // Unwrap the Result - both Ok and Err contain HttpResponse
                let http_response = response.unwrap_or_else(|e| e);
                http_response.into_hyper()
            } else {
                // No fallback defined, return default 404
                HttpResponse::text("404 Not Found").status(404).into_hyper()
            }
        }
    };

    response
}

/// Built-in health check endpoint at /_ferro/health
/// Returns {"status": "ok", "timestamp": "..."} by default
/// Add ?db=true to also check database connectivity (/_ferro/health?db=true)
async fn health_response(query: &str) -> hyper::Response<Full<Bytes>> {
    use chrono::Utc;
    use serde_json::json;

    let timestamp = Utc::now().to_rfc3339();
    let check_db = query.contains("db=true");

    let mut response = json!({
        "status": "ok",
        "timestamp": timestamp
    });

    if check_db {
        // Try to check database connection
        match check_database_health().await {
            Ok(_) => {
                response["database"] = json!("connected");
            }
            Err(e) => {
                response["database"] = json!("error");
                response["database_error"] = json!(e);
            }
        }
    }

    let body =
        serde_json::to_string(&response).unwrap_or_else(|_| r#"{"status":"ok"}"#.to_string());

    hyper::Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap()
}

/// Serve the pre-built ferro-json-ui base CSS.
///
/// The bytes are embedded at compile time via ferro_json_ui::FERRO_BASE_CSS.
/// Response: 200, text/css, 24h cache. No user input reaches this handler —
/// the match arm is an exact string, and the body is static framework content.
fn serve_ferro_base_css() -> hyper::Response<Full<Bytes>> {
    let css = ferro_json_ui::FERRO_BASE_CSS;
    hyper::Response::builder()
        .status(200)
        .header("Content-Type", "text/css; charset=utf-8")
        .header("Content-Length", css.len().to_string())
        .header("Cache-Control", "public, max-age=86400")
        .body(Full::new(Bytes::from_static(css.as_bytes())))
        .unwrap()
}

/// Check database health by attempting a simple query
async fn check_database_health() -> Result<(), String> {
    use crate::database::DB;
    use sea_orm::ConnectionTrait;

    if !DB::is_connected() {
        return Err("Database not initialized".to_string());
    }

    let conn = DB::connection().map_err(|e| e.to_string())?;

    // Execute a simple query to verify connection is alive
    conn.inner()
        .execute_unprepared("SELECT 1")
        .await
        .map_err(|e| format!("Database query failed: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod ferro_base_css_route_tests {
    use super::*;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn serve_ferro_base_css_returns_200_with_text_css_content_type() {
        let response = serve_ferro_base_css();

        assert_eq!(response.status(), 200, "expected 200 OK");

        let ct = response
            .headers()
            .get("Content-Type")
            .expect("Content-Type header missing")
            .to_str()
            .unwrap();
        assert_eq!(ct, "text/css; charset=utf-8");

        let cc = response
            .headers()
            .get("Cache-Control")
            .expect("Cache-Control header missing")
            .to_str()
            .unwrap();
        assert_eq!(cc, "public, max-age=86400");

        let cl = response
            .headers()
            .get("Content-Length")
            .expect("Content-Length header missing")
            .to_str()
            .unwrap()
            .parse::<usize>()
            .expect("Content-Length must be an integer");
        assert_eq!(cl, ferro_json_ui::FERRO_BASE_CSS.len());
    }

    #[tokio::test]
    async fn serve_ferro_base_css_body_equals_embedded_constant() {
        let response = serve_ferro_base_css();
        let body_bytes = response
            .into_body()
            .collect()
            .await
            .expect("body collect")
            .to_bytes();
        assert_eq!(
            body_bytes.as_ref(),
            ferro_json_ui::FERRO_BASE_CSS.as_bytes()
        );
        assert!(!body_bytes.is_empty());
    }
}
