use std::{
    io::{ErrorKind, Result},
    sync::Arc,
};

use app::{App, serve::serve};
use tokio_ext::{net, runtime, spawn};
use tokio_util::{compat::TokioAsyncWriteCompatExt, io};

/// Boots the application using the `tokio` asynchronous runtime
///
/// Builds a multi-threaded `tokio` runtime, binds to a socket, starts a tcp listener and starts
/// listening for incoming connections. Spawns a new asynchronous task per incoming connection,
/// and passes the read stream and write handle to `snx`.
pub fn runtime(app: Arc<App>) -> Result<()> {
    runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let listener = net::TcpListener::bind("127.0.0.1:2002").await?;
            loop {
                match listener.accept().await {
                    Ok((socket, _)) => {
                        let (rx, tx) = socket.into_split();
                        let app = app.clone();
                        spawn(async move {
                            serve(&app, io::ReaderStream::new(rx), tx.compat_write()).await
                        });
                    }
                    Err(err) => match err.kind() {
                        ErrorKind::PermissionDenied | ErrorKind::OutOfMemory => tracing::error!(target = "snx:server", error = %err, "critical resource limit or permission failure on listener"),
                        _ => tracing::debug!(target = "snx::server", error = %err, "failed to accept incoming connection"),
                    },
                }
            }
        })
}
