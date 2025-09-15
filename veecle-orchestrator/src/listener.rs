use camino::{Utf8Path, Utf8PathBuf};
use eyre::WrapErr;
use tokio::net::UnixStream;

/// A wrapper for [`tokio::net::UnixListener`] that handles removing the socket from the filesystem on drop, see that
/// for API docs.
#[derive(Debug)]
pub(crate) struct UnixListener {
    listener: tokio::net::UnixListener,
    path: Utf8PathBuf,
}

impl UnixListener {
    pub(crate) fn bind(path: impl AsRef<Utf8Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_owned();
        Ok(Self {
            listener: tokio::net::UnixListener::bind(&path)?,
            path,
        })
    }

    pub(crate) async fn accept(&self) -> eyre::Result<UnixStream> {
        let (stream, _addr) = self.listener.accept().await.wrap_err("accepting client")?;
        Ok(stream)
    }
}

impl Drop for UnixListener {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_file(&self.path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::error!(?error, ?self.path, "error removing unix socket");
        }
    }
}
