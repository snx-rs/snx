# roadmap and milestones

## 0.0.3 | core functionality

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

## 0.0.5 | json io and database

- [x] shared app context for handlers and middleware (db connection and config)
- [x] ergonomic json response body writing
- [x] ergonomic json request body reading
- [x] diesel integration

## 0.0.6 session and cookies

- [ ] pass data from middleware to middleware/handler
- [x] session storage
- [x] cookies

## 0.0.7 templating, forms and validation

- [ ] validation
- [ ] templating engine
- [ ] form request body parsing/reading (with validation)
- [ ] ergonomic redirects
- [ ] static file (dir) serving (maybe)
- [ ] csrf

## uncategorized

- custom (user-defined) shared context for all handlers/middleware
- ranges
- streaming responses
- request pipelining
- transfer encoding
- content encoding
- websockets
- server-sent events
- easy creating of CRUD (API) routes and controllers (api resources)
- authentication
- cors
- flash messages
- jwt
