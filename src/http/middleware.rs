use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use jiff::Zoned;

use crate::Context;

use super::{
    request::Request,
    response::{IntoResponse, Response},
};

pub type MiddlewareHandler = Arc<
    Box<
        dyn Fn(Context, Request, Box<dyn Fn(Request) -> Response>) -> Box<dyn IntoResponse>
            + Send
            + Sync,
    >,
>;

/// Built-in middleware to trace requests.
pub fn trace_requests(
    _: Context,
    req: Request,
    next: Box<dyn Fn(Request) -> Response>,
) -> Box<dyn IntoResponse> {
    let now = SystemTime::now();

    let res = next(req.clone());

    let elapsed = now.elapsed().unwrap();

    tracing::info!(
        "{} {} \"{} {}\" {} {}B {}ms",
        req.headers().get("host").unwrap_or("-".to_string()),
        req.peer_addr().map(|p| p.to_string()).unwrap_or_default(),
        req.method(),
        req.path(),
        res.status(),
        res.body().clone().unwrap_or_default().len(),
        elapsed.as_millis(),
    );

    Box::new(res)
}

/// Built-in cookie-based middleware to initialize sessions.
///
/// Retrieves the current session if it exists otherwise starts a new session and adds the session
/// to the request.
#[cfg(feature = "sessions")]
pub fn initialize_session(
    ctx: Context,
    mut req: Request,
    next: Box<dyn Fn(Request) -> Response>,
) -> Box<dyn IntoResponse> {
    if let Some(session_store) = ctx.session_store {
        if let Some(cookie) = req.cookies().get(
            &ctx.config
                .session
                .clone()
                .unwrap_or_default()
                .cookie_key
                .unwrap_or("snx-session".to_string()),
        ) {
            if let Ok(id) = cookie.value().parse::<u128>() {
                let mut guard = session_store.try_lock().unwrap();
                if let Ok(Some(session)) = guard.load(id) {
                    if session.expires_at > Zoned::now() {
                        drop(guard);
                        req.session = Some(session);
                        return Box::new(next(req));
                    }

                    guard.delete(session.id).unwrap();
                }
            }
        }

        let duration = crate::config::parse_duration(
            &ctx.config
                .session
                .unwrap_or_default()
                .expires_after
                .unwrap_or("7d".to_string()),
        )
        .unwrap();
        let session = crate::session::Session::new(
            Zoned::now().checked_add(duration).unwrap(),
            session_store.clone(),
        );

        session_store
            .try_lock()
            .unwrap()
            .create(session.clone())
            .unwrap();
        req.session = Some(session.clone());

        let mut cookies = biscotti::ResponseCookies::new();
        cookies.insert(biscotti::ResponseCookie::new(
            "snx-session",
            session.id.to_string(),
        ));

        return Box::new((cookies, next(req)));
    }

    Box::new(next(req))
}
