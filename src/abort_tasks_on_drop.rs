use tokio::task::{AbortHandle, JoinHandle};

/// Allows limiting the runtime of a tokio task to a lifetime.
pub struct AbortTaskOnDrop {
    abort_handle: AbortHandle,
}

impl<T> From<JoinHandle<T>> for AbortTaskOnDrop {
    fn from(join_handle: JoinHandle<T>) -> Self {
        Self {
            abort_handle: join_handle.abort_handle(),
        }
    }
}

impl Drop for AbortTaskOnDrop {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}
