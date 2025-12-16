pub trait ThreadScope<'s> {
    fn spawn<F>(&self, f: F) where F: FnOnce() + Send + 's;
}

pub trait ThreadProvider {
    type Scope<'scope, 'env>: ThreadScope<'scope> where Self: 'env;

    fn scope<'env, F, R>(&self, f: F) -> R where F: for<'scope> FnOnce(&Self::Scope<'scope, 'env>) -> R + Send;
}

pub struct StdThreadProvider;

pub struct StdScope<'scope, 'env> {
    inner: &'scope std::thread::Scope<'scope, 'env>,
}

impl<'scope, 'env> ThreadScope<'scope> for StdScope<'scope, 'env> {
    fn spawn<F>(&self, f: F) where F: FnOnce() + Send + 'scope {
        let _ = self.inner.spawn(f);
    }
}

impl ThreadProvider for StdThreadProvider {
    type Scope<'scope, 'env> = StdScope<'scope, 'env> where Self: 'env;

    fn scope<'env, F, R>(&self, f: F) -> R where F: for<'scope> FnOnce(&Self::Scope<'scope, 'env>) -> R + Send {
        std::thread::scope(|s| {
            let scope = StdScope { inner: s };
            f(&scope)
        })
    }
}
