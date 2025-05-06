# snx

snx is an experimental, opiniated and batteries-included web framework that allows you to quickly develop robust web applications using Rust.

## overview of features

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
                    .post("/", store_tenant)
                    .get("/", show_tenants)
                    .get("/{id}", show_tenant)
                    .post("/{id}", update_tenant)
                    .delete("/{id}", delete_tenant)
            })
    })
    .build()
    .unwrap()
```

###### handlers and middleware

handlers in snx are functions or closures which take 2 arguments (a context and
a request) and produce anything that can be turned into a response. the first
argument can be used to interact with parts of your applications, for example,
executing database queries, sending emails or rendering templates. the second
argument contains all the request information and allows you to read incoming
data from the request and act on it accordingly.

```rust
#[derive(Deserialize, Insertable)]
#[diesel(table_name = crate::schema::tenants)]
struct StoreTenantPayload {
    name: String,
}

fn store_tenant(ctx: Context, req: Request) -> Result<(StatusCode, Json<Tenant>)> {
    let payload = req.json::<StoreTenantPayload>()?;
    let tenant = payload
        .insert_into(tenants)
        .get_result::<Tenant>(&mut ctx.db.get().unwrap())?;

    Ok((StatusCode::Created, Json(tenant)))
}
```

middleware in snx are almost exactly like handlers but they take 3 arguments (a
context, a request and a next function). the third argument is used to call the
next middleware/handler in the chain. middleware are layered like an onion, just
like axum.

###### templating

you can define templates using the `template` macro. this macro will be
transpiled to a `format` macro call at build-time, which will result in a
string at run-time. "components" are just functions/closures which return a
string.

- use braced blocks to write arbitrary Rust code
- braces can be omitted for attribute values
- quoted and unquoted text nodes
- fragments

```rust
fn article(article: Article) -> String {
    template! {
        <a href=format!("articles/{}", article.slug)>
            <article>
                <img
                    src=article.cover.url
                    alt=article.cover.alt
                    width="400"
                    height="200"
                />
                <h2>{article.title}</h2>
                <small>author: {article.author}</small>
            </article>
        </a>
    } 
}

fn index(ctx: Context, _: Request) -> Result<Html> {
    let articles = articles
        .select(Article::as_select())
        .load(&mut ctx.db.get().unwrap())?;

    Ok(Html(layout(template! {
        <section>
            <h1>articles</h1>
            <ul>
                {articles.into_iter().map(article).collect::<Vec<String>>().join("")} 
            </ul>
        </section>
    })))
}
```

## non-features

###### HTTP/2, HTTP/3 and TLS/SSL

snx is designed to sit behind a reverse proxy and only supports HTTP/1.1 without
TLS/SSL. configure a reverse proxy (e.g. nginx or Caddy) for HTTP/2, HTTP/3 and
TLS/SSL.
