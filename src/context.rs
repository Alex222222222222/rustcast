use tokio::sync::Mutex;

struct ContextInner {
    id: usize,
}

pub struct Context(Mutex<ContextInner>);

pub static CONTEXT: once_cell::sync::Lazy<Context> =
    once_cell::sync::Lazy::new(|| Context(Mutex::new(ContextInner { id: 1 })));

impl Context {
    pub async fn get_id(&self) -> usize {
        let mut inner = self.0.lock().await;
        let id = inner.id;
        inner.id += 1;
        id
    }
}
