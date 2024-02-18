#![allow(unreachable_patterns)]

use cursive_core::{
    theme::ColorStyle, utils::markup::StyledString, view::ViewWrapper, wrap_impl, CbSink, Printer,
    Vec2, View,
};
use std::ops::RangeInclusive;

use crate::InfiniteAsyncView;

pub enum LoadingCharStyle {
    FullBlock,
    Dash,
    DashAndEquals,
    ReverseDashAndEquals,
}

pub struct LoadingStateView {
    state: usize,
    style: LoadingCharStyle,
}

impl LoadingStateView {
    pub fn new(style: LoadingCharStyle, reverse: bool) -> Self {
        Self {
            state: if !reverse { 0 } else { usize::MAX },
            style,
        }
    }

    /// Given a state, the length of the middle, and the total length, returns two sets of ranges (tuples), representing the start and end of the coloured and uncoloured parts of the loading bar.
    // Formula given as:
    /* Full formula to determine the ranges in which to change apperance for a progress bar, given the state (s), length of the middle (n), and total length (t):
    If s < t:
        EndIndex = s.
        StartIndex = (s - n) + 1.
    Else:
        EndIndex = s % t.
        StartIndex = (EndIndex - n) % t.
        If EndIndex < StartIndex:
            Range1 = StartIndex to t.
            Range2 = 1 to EndIndex.
         */
    /// Output is a tuple of two tuples. The first tuple is the start and end of the coloured part. The second tuple is the start and end of the uncoloured part.
    fn calculate_ranges(
        state: usize,
        middle: usize,
        total_length: usize,
    ) -> (RangeInclusive<usize>, RangeInclusive<usize>) {
        let (range_1, range_2) = if state < total_length {
            let end = state % total_length;
            let start = (state).saturating_sub(middle);
            (start..=end, end..=start)
        } else {
            let end = state % total_length;
            let start = (end + total_length).saturating_sub(middle) % total_length;

            // Start and end are now ranges for the colourepart.
            // However, the reason we have to is because it's not capable of looping around.
            // We have to bascially split the thing up into two, if the end is less than the start.
            if end < start {
                let range_1 = start..=total_length;
                let range_2 = 1..=end;
                return (range_1, range_2);
            }

            (start..=end, end..=start)
        };

        (range_1, range_2)
    }

    /// Given a constaint of minimum 3x1, Returns the length of the padding characters on the outside, and the length of the coloured characters in the middle.
    // Formula as follows:
    /*If:
    n % 3 = 2 - Difference between inner and outer is 2.
        Do n += 1.
        Then middle = n / 3.
        Then outer = middle - 2.
    n % 3 = 1 - Difference between outer and inner is 1.
        Do n += 2.
        Then middle = n / 3 + 1.
        Then outer = middle - 1
    n % 3 = 0 - No difference. Equal values.
        Do n / 3 to get lengths.
        */
    /// Output is done (middle, outer)
    fn calculate_length(constraint: Vec2) -> (usize, usize) {
        // Get the total length we can get from the constaint. And split it into thirds.
        let total_length = constraint.x;
        let (mid, out) = match total_length % 3 {
            2 => {
                let middle = (total_length + 1) / 3;
                let outer = middle - 2;
                (middle, outer)
            }
            1 => {
                let middle = (total_length + 2) / 3 + 1;
                let outer = middle - 1;
                (middle, outer)
            }
            0 => {
                let middle = total_length / 3;
                (middle, middle)
            }
            _ => unreachable!(),
        };

        (mid, out)
    }

    fn calculate_string(&self, constraint: Vec2) -> StyledString {
        // Return type is a string representing the loading bar.
        // The vec2 is the constaint, which is always a minimum of 3x1. And always has a height of 1.
        // State is just a variable to represent where we'd be.
        // We get the total length we can get from the constaint. And split it into thirds.
        // E.g. 3 length. 1, 1, 1. 4 length. 1, 2, 1. 5 length. 1, 3, 1. 6 length. 2, 2, 2. 7 length. 2, 3, 2.  8 length. 2, 4, 2. etc
        // The middle variable represents how many chars long the thicker/coloured part of the bar is.
        // The first and last variable represents how many chars long the thinner/uncoloured part of the bar is. They're also always the same.
        // We use the state variable to represent where we are in the loading bar.
        // In example, if state is 0, then the first char should be coloured, and the rest should be uncoloured.
        // If state is 1, then the first 2 chars have the chance to be coloured, depending on the length of the middle.
        // What we use state for is to calcualte where the coloured should be.

        // Check it's at least 3x1, if not, we assume it's 3x1.
        let constraint = if constraint.x < 3 {
            Vec2::new(3, 1)
        } else {
            constraint
        };

        let (middle, _outer) = Self::calculate_length(constraint);
        let (range_1, range_2) = Self::calculate_ranges(self.state, middle, constraint.x);
        // log::error!("{:?} {:?}", range_1, range_2);

        let mut string = StyledString::new();

        let (normal, inactive) = match self.style {
            LoadingCharStyle::FullBlock => ("█", "█"),
            LoadingCharStyle::Dash => ("━", "━"),
            LoadingCharStyle::DashAndEquals => ("━", "="),
            LoadingCharStyle::ReverseDashAndEquals => ("=", "━"),
            _ => (" ", " "),
        };

        for i in 1..=constraint.x {
            // If the range contains the current index, then we add a coloured character.
            // log::error!(
            //     "{} {:?} {:?}",
            //     i,
            //     range_1.contains(&i),
            //     range_2.contains(&i)
            // );
            if range_1.contains(&i) || range_2.contains(&i) {
                // Add a coloured character
                string.append_styled(normal, ColorStyle::secondary());
            }
            // If the range doesn't contain the current index, then we add an uncoloured character.
            else {
                // Add an uncoloured character
                string.append_plain(inactive);
            }
        }

        string
    }
}

impl View for LoadingStateView {
    fn draw(&self, printer: &Printer) {
        let string = self.calculate_string(printer.size);
        printer.print_styled((0, 0), &string);
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        Self::calculate_length(constraint).into()
    }
}

pub struct LoadingView {
    infinite_view: InfiniteAsyncView<LoadingStateView>,
}

impl LoadingView {
    pub fn new(s: CbSink, style: LoadingCharStyle, reverse: bool) -> Self {
        let view = LoadingStateView::new(style, reverse);
        let infinite_view = InfiniteAsyncView::new(
            s,
            move |view| async move {
                // Increase state
                let mut view = view.lock().await;
                if reverse {
                    view.state -= 1;
                    if view.state == usize::MIN + 1 {
                        view.state = usize::MAX;
                    }
                } else {
                    view.state += 1;
                    if view.state == usize::MAX - 1 {
                        view.state = 0;
                    }
                }
            },
            view,
            80,
        );
        Self { infinite_view }
    }
}

impl ViewWrapper for LoadingView {
    wrap_impl!(self.infinite_view: InfiniteAsyncView<LoadingStateView>);
}
