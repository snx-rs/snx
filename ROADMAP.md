# roadmap and milestones

## 0.0.3 | core functionality and validating the idea

- [x] basic non-async http/1.1 server
- [x] flexible router (with dynamic route segments, wildcards, methods, prefixes and middleware groups)
- [x] ergonomic route handlers
- [x] ergonomic responses with status codes, headers and bodies (with composition)
- [x] panic handling and application tracing
- [x] config toml file
- [x] middleware
- [x] built-in middleware for request/response tracing

## 0.0.4 | improved http handling and (subdomain) routing

- [x] ergonomic headers
- [x] content-type and encoding header for string responses
- [x] date header
- [ ] add support for chunked transfer encoding
- [ ] streaming responses
- [ ] lazy body reading using request object
- [ ] (sub)domain routing

## 0.0.5 | json io and database (milestone: very basic working REST JSON API)

- [ ] shared context for all handlers/middleware
- [ ] diesel integration
- [ ] ergonomic json response body writing (content type)
- [ ] ergonomic json request body reading
- [ ] respond with 415 when incoming data is not expected format
- [ ] error response types
