use cursive_core::{CbSink, Printer, View};
use std::{future::Future, sync::Arc};
use tokio::sync::{watch, Mutex};

/// Similar to AsyncView, but it's always running the closure.
/// Has a mutable reference to the view, so it can be updated at any time.
/// Note: The async closure will automatically be closed, when the view is dropped.
pub struct InfiniteAsyncView<T>
where
    T: View + 'static,
{
    view: Arc<Mutex<T>>,
    tx: watch::Sender<bool>,
}

impl<T> InfiniteAsyncView<T>
where
    T: View + 'static,
{
    /// The interval is how many milliseconds after executing the closure it should execute.
    /// This is to limit it...  
    /// The returned view in the closure, is whatever the initial view is.
    pub fn new<F, Fut>(s: CbSink, cb: F, init: T, interval: usize) -> Self
    where
        F: (FnMut(Arc<Mutex<T>>) -> Fut) + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let view = Arc::new(Mutex::new(init));
        let cloned = view.clone();
        let (tx, rx) = watch::channel(false);

        let mut callback = cb;

        tokio::spawn(async move {
            loop {
                // If rx is true, we break out of the loop
                if *rx.borrow() {
                    break;
                }

                let loop_cloned: Arc<Mutex<T>> = cloned.clone();
                // We execute the closure every tick (which is tickrate/1000 ms)
                // Get a write reference
                {
                    // let mut view = loop_cloned.lock().await;
                    // let view = view.deref_mut();

                    callback(loop_cloned).await;
                }

                // Execute noop to update the view
                s.send(Box::new(|s| s.noop())).unwrap();

                tokio::time::sleep(tokio::time::Duration::from_millis(interval as u64)).await;
            }
        });

        Self { view, tx }
    }
}

impl<T> View for InfiniteAsyncView<T>
where
    T: View + 'static,
{
    fn draw(&self, printer: &Printer) {
        self.view.try_lock().unwrap().draw(printer);
    }

    fn layout(&mut self, size: cursive_core::Vec2) {
        self.view.try_lock().unwrap().layout(size);
    }

    fn required_size(&mut self, constraint: cursive_core::Vec2) -> cursive_core::Vec2 {
        self.view.try_lock().unwrap().required_size(constraint)
    }
}

impl<T> Drop for InfiniteAsyncView<T>
where
    T: View + 'static,
{
    fn drop(&mut self) {
        let _ = self.tx.send(true);
    }
}
