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

## 0.0.4 | improved http handling and more routing options

- [x] (sub)domain routing
- [x] respond with 405 when incorrect method is used
- [x] date header
- [x] body reading (raw and string) and reading/writing content-length
- [x] ergonomic header reading/writing
- [x] html responses

## 0.0.5 | json io, forms, validation and database

- [ ] shared app context for handlers and middleware (db connection, config, etc.)
- [ ] ergonomic json response body writing
- [ ] ergonomic json request body reading
- [ ] form json request body reading
- [ ] validation
- [ ] diesel integration
- [ ] flash messages
- [ ] easy creating of CRUD (API) routes and controllers

## 0.0.6 templating & forms

- [ ] engine-agnostic templating
- [ ] form request body parsing/reading (with validation)
- [ ] ergonomic redirects

## uncategorized

- custom (user-defined) shared context for all handlers/middleware
- ranges
- streaming responses
- request pipelining
- transfer encoding
- content encoding
- websockets
- server-sent events
- static file (dir) serving
