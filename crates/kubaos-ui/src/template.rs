//! Templates: a screen's layout, empty of any app's content. Its regions, how it reflows as its
//! window resizes, how it moves between its pages and where the keyboard goes; the app fills it
//! with organisms built from the components.
//!
//! # Owns
//!
//! - An app's window as a sidebar beside a scrolling page: [`sidebar_page`].
//!
//! # Not here
//!
//! - The components a template is built from: [`crate::component`] and [`crate::layout`].

pub mod sidebar_page;
