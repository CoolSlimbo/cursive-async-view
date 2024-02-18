mod infinite;
mod progress;
mod utils;

pub use infinite::{default_animation, default_error, AnimationFrame, AsyncState, AsyncView};
pub use progress::{
    default_progress, default_progress_error, AnimationProgressFrame, AsyncProgressState,
    AsyncProgressView,
};

doc_comment::doctest!("../../README.md");
