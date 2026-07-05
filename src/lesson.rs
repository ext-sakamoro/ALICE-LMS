//! lesson.

use crate::ids::*;

// Lesson
// ---------------------------------------------------------------------------

/// A single lesson inside a module.
#[derive(Debug, Clone)]
pub struct Lesson {
    pub id: LessonId,
    pub title: String,
    pub order: u32,
    pub duration_minutes: u32,
}

impl Lesson {
    #[must_use]
    pub fn new(id: LessonId, title: &str, order: u32, duration_minutes: u32) -> Self {
        Self {
            id,
            title: title.to_owned(),
            order,
            duration_minutes,
        }
    }
}
