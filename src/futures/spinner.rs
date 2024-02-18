use std::{collections::HashMap, str::FromStr};

use crate::InfiniteAsyncView;
use cursive_core::{
    utils::markup::StyledString, view::ViewWrapper, wrap_impl, CbSink, Printer, Vec2, View,
};
use spinners::{utils::spinners_data::SPINNERS, Spinners};

fn transform_spinners() -> HashMap<Spinners, (Vec<StyledString>, usize)> {
    let mut frames = HashMap::new();

    for (id, spinner) in SPINNERS.iter() {
        // For each spinner, we create a new vector of StyledString
        let mut spinner_frames = Vec::new();
        for frame in spinner.frames.iter() {
            let styled_frame = StyledString::plain(*frame);
            spinner_frames.push(styled_frame);
        }
        frames.insert(
            Spinners::from_str(id.as_str()).unwrap(),
            (spinner_frames, spinner.interval as usize),
        );
    }

    frames
}

pub struct SpinnerStateView {
    frame: usize,
    frames: Vec<StyledString>,
    total_frames: usize,
}

impl SpinnerStateView {
    pub fn new(style: Spinners) -> (Self, usize) {
        let spinners = transform_spinners();
        let (frames, interval) = spinners.get(&style).unwrap();

        let len = frames.len();

        (
            Self {
                frame: 0,
                frames: frames.clone(),
                total_frames: len,
            },
            *interval,
        )
    }
}

impl View for SpinnerStateView {
    fn draw(&self, printer: &Printer) {
        // Get the current frame
        let frame = &self.frames[self.frame];
        // Draw the frame
        printer.print_styled((0, 0), frame);
    }

    fn required_size(&mut self, _constraint: Vec2) -> Vec2 {
        Vec2::new(self.frames[self.frame].width(), 1)
    }
}

pub struct SpinnerView {
    infinite_view: InfiniteAsyncView<SpinnerStateView>,
}

impl SpinnerView {
    pub fn new(s: CbSink, style: Spinners) -> Self {
        let (view, interval) = SpinnerStateView::new(style);
        let infinite_view = InfiniteAsyncView::new(
            s,
            move |view| async move {
                let mut view = view.lock().await;
                // Add one to frame, if it's greater than total_frames, reset it to 0
                view.frame = (view.frame + 1) % view.total_frames;

                // log::error!("Frame: {}", view.frame);
            },
            view,
            interval,
        );

        Self { infinite_view }
    }
}

impl ViewWrapper for SpinnerView {
    wrap_impl!(self.infinite_view: InfiniteAsyncView<SpinnerStateView>);
}
