use std::convert::identity;

use crate::{App, router};

#[derive(Default)]
pub struct Builder {
    router: Option<Box<dyn Fn(router::Builder) -> router::Builder>>,
}

impl Builder {
    /// Defines the application's routes
    pub fn with_routes<F>(mut self, f: F) -> Self
    where
        F: Fn(router::Builder) -> router::Builder + 'static,
    {
        self.router = Some(Box::new(f));
        self
    }

    /// Builds the application
    pub fn build(self) -> Result<App, BuildError> {
        let router = router::Builder::new("localhost".to_string());
        Ok(App {
            router: self.router.unwrap_or_else(|| Box::new(identity))(router).build()?,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    #[error(transparent)]
    RouterInsert(#[from] matchit::InsertError),
}
