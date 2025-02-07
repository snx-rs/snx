# snx

snx is an experimental batteries-included web framework for Rust.

## design goals

snx is designed to fill the gap of non-async batteries-included web frameworks
in the Rust ecosystem.

## high level features

###### non-async

snx does not use async Rust at all and achieves asynchronous execution using
threading. snx not using async has numerious benefits like: not being locked
into an async runtime's ecosystem such as `tokio`, not requiring an async
runtime at all and not having to manage the added complexity of async as a whole
allowing you to focus on your application and domain logic rather than fighting
over lifetimes.

this does come with a couple of trade-offs, namely ... TBA

###### flexible routing

snx provides a fast, ergonomic and macro-free routing system based on `matchit`
that supports dynamic route segments, wildcards, prefixes and middleware.

```rust
Router::builder()
    .prefix("/users", |router| {
        router
            .post("/", store_user)
            .get("/", list_users)
            .get("/{id}", get_user)
            .put("/{id}", update_user)
            .delete("/{id}", delete_user)
    })
    .build()
    .unwrap()
```

## non-features

###### HTTP/2, HTTP/3 and TLS/SSL

snx is designed to sit behind a reverse proxy and thus only supports HTTP/1.1
without TLS/SSL. configure a reverse proxy (e.g. nginx or Caddy) for HTTP/2, HTTP/3
and TLS/SSL.
