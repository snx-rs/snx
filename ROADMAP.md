# roadmap

- 0.0.2 ✅
    - simple synchronous http/1.1 server ✅
    - router with support for dynamic parameters ✅
    - very simple route handlers (only status + headers, no body) ✅
    - simple request logging ✅

- 0.0.3 ❌
    - router with support for optional parameters, wildcards and regex parameters ❌
    - axum-like extractors for handlers ❌
    - ergonomic responses (with a body and content-types) and error handling for handlers ❌
    - application wide good error handling (tracing?) ❌
    - custom .env configuration (type-checked) ❌

- 0.0.4 ❌
    - router middleware, route groups (prefix, middleware) ❌
    - route names + compile time validation using macros (e.g. route!('posts.view', 5)) ❌
    - diesel integration ❌
