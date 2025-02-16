# snx

snx is an experimental batteries-included web framework for Rust.

## design goals

snx is designed to fill the gap of non-async batteries-included web frameworks
in the Rust ecosystem.

we try to keep the amount of dependencies used by snx to a minimum.

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
that supports dynamic route segments, wildcards, prefixes, middleware and
hostname-based routing.

```rust
Router::builder()
    .get("/", show_index)
    .get("/contact", show_contact)
    .post("/contact", submit_contact)
    .host("{tenant}.acme.com", |builder| {
        builder
            .get("/", show_tenant_index)
            .get("/media/{*path}", retrieve_tenant_media)
    })
    .middleware(&[auth], |builder| {
        builder
            .prefix("/dashboard/tenants", |builder| {
                builder
                    .get("/create", show_create_tenant)
                    .post("/", store_tenant)
                    .get("/", show_tenants)
                    .get("/{id}", show_tenant)
                    .get("/{id}/edit", show_edit_tenant)
                    .post("/{id}", update_tenant)
                    .delete("/{id}", delete_tenant)
            })
    })
    .build()
    .unwrap()
```

## non-features

###### HTTP/2, HTTP/3 and TLS/SSL

snx is designed to sit behind a reverse proxy and thus only supports HTTP/1.1
without TLS/SSL. configure a reverse proxy (e.g. nginx or Caddy) for HTTP/2, HTTP/3
and TLS/SSL.
