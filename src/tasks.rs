use bevy::tasks::{AsyncComputeTaskPool, Task};

pub fn spawn_async<F, T>(f: F) -> Task<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    AsyncComputeTaskPool::get().spawn(async move { f() })
}
