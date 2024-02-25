#![doc(html_logo_url = "https://raw.githubusercontent.com/griffi-gh/hui/master/.assets/hui.svg")]
//!
//! Simple UI library for games and other interactive applications
//!
//! # Features
#![doc = document_features::document_features!()]

#![allow(unused_parens)]
#![forbid(unsafe_code)]
#![forbid(unsafe_op_in_unsafe_fn)]

mod instance;
pub mod layout;
pub mod rectangle;
pub mod background;
pub mod element;
pub mod event;
pub mod input;
pub mod draw;
pub mod measure;
pub mod state;
pub mod text;

pub use instance::UiInstance;
