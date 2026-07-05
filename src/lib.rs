//! ALICE-LMS: Learning Management System.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::module_name_repetitions,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::too_many_lines,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::return_self_not_must_use
)]

pub mod certificate;
pub mod course;
pub mod enrollment;
pub mod grading;
pub mod ids;
pub mod lesson;
pub mod module;
pub mod prelude;
pub mod progress;
pub mod quiz;
pub mod signed_certificate;
pub mod system;

#[cfg(test)]
mod integration_tests;

pub use crate::certificate::*;
pub use crate::course::*;
pub use crate::enrollment::*;
pub use crate::grading::*;
pub use crate::ids::*;
pub use crate::lesson::*;
pub use crate::module::*;
pub use crate::progress::*;
pub use crate::quiz::*;
pub use crate::system::*;
