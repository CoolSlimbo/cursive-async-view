use cursive_core::{Cursive, Printer, View};
use std::{
    future::Future,
    sync::{Arc, Mutex},
};

pub struct AsyncView<T, P> {
    pending_view: Arc<Mutex<Option<P>>>,
    display_view: Arc<Mutex<Option<T>>>,
}

impl<T, P> AsyncView<T, P>
where
    T: View + 'static,
    P: View + 'static,
{
    pub fn new<F>(s: &mut Cursive, cb: F, view: P) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        let display = Arc::new(Mutex::new(None));
        let cloned = display.clone();

        let pending = Arc::new(Mutex::new(Some(view)));
        let view = pending.clone();

        let sink = s.cb_sink().clone();

        tokio::spawn(async move {
            let result = cb.await;
            display.lock().unwrap().replace(result);
            // Drop the pending view
            view.lock().unwrap().take();
            sink.send(Box::new(|s| s.noop())).unwrap(); // For some reason, noop causes it to update
        });

        Self {
            pending_view: pending,
            display_view: cloned,
        }
    }
}

impl<T, P> View for AsyncView<T, P>
where
    T: View + 'static,
    P: View + 'static,
{
    fn draw(&self, printer: &Printer) {
        if let Some(view) = self.display_view.lock().unwrap().as_ref() {
            view.draw(printer);
            return;
        }
        if let Some(view) = self.pending_view.lock().unwrap().as_ref() {
            view.draw(printer);
            return;
        }
    }

    fn layout(&mut self, size: cursive_core::Vec2) {
        if let Some(view) = self.display_view.lock().unwrap().as_mut() {
            view.layout(size);
            return;
        }
        if let Some(view) = self.pending_view.lock().unwrap().as_mut() {
            view.layout(size);
            return;
        }
    }

    fn needs_relayout(&self) -> bool {
        if let Some(view) = self.display_view.lock().unwrap().as_ref() {
            return view.needs_relayout();
        }
        if let Some(view) = self.pending_view.lock().unwrap().as_ref() {
            return view.needs_relayout();
        }
        true
    }

    fn required_size(&mut self, constraint: cursive_core::Vec2) -> cursive_core::Vec2 {
        if let Some(view) = self.display_view.lock().unwrap().as_mut() {
            return view.required_size(constraint);
        }
        if let Some(view) = self.pending_view.lock().unwrap().as_mut() {
            return view.required_size(constraint);
        }
        cursive_core::Vec2::zero()
    }

    // I'll impliment them all later ig lol
}
