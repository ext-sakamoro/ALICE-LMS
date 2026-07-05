//! module.

use crate::ids::*;
use crate::lesson::Lesson;

// Module
// ---------------------------------------------------------------------------

/// A module groups related lessons inside a course.
#[derive(Debug, Clone)]
pub struct Module {
    pub id: ModuleId,
    pub title: String,
    pub order: u32,
    pub lessons: Vec<Lesson>,
}

impl Module {
    #[must_use]
    pub fn new(id: ModuleId, title: &str, order: u32) -> Self {
        Self {
            id,
            title: title.to_owned(),
            order,
            lessons: Vec::new(),
        }
    }

    pub fn add_lesson(&mut self, lesson: Lesson) {
        self.lessons.push(lesson);
        self.lessons.sort_by_key(|l| l.order);
    }

    #[must_use]
    pub fn total_duration(&self) -> u32 {
        self.lessons.iter().map(|l| l.duration_minutes).sum()
    }

    #[must_use]
    pub const fn lesson_count(&self) -> usize {
        self.lessons.len()
    }
}
