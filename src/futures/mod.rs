// This section of the library is gonna take an entirely new approach to handling AsyncViews, utlilsing futures
mod infinite;
mod loading;
mod spinner;
mod view;

pub use infinite::InfiniteAsyncView;
pub use loading::{LoadingCharStyle, LoadingView};
pub use spinner::SpinnerView;
pub use spinners::Spinners as SpinnerStyle;
pub use view::AsyncView;
