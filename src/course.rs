//! course.

use crate::ids::*;
use crate::module::Module;

// Course
// ---------------------------------------------------------------------------

/// A course contains ordered modules.
#[derive(Debug, Clone)]
pub struct Course {
    pub id: CourseId,
    pub title: String,
    pub description: String,
    pub modules: Vec<Module>,
}

impl Course {
    #[must_use]
    pub fn new(id: CourseId, title: &str, description: &str) -> Self {
        Self {
            id,
            title: title.to_owned(),
            description: description.to_owned(),
            modules: Vec::new(),
        }
    }

    pub fn add_module(&mut self, module: Module) {
        self.modules.push(module);
        self.modules.sort_by_key(|m| m.order);
    }

    #[must_use]
    pub fn total_lessons(&self) -> usize {
        self.modules.iter().map(Module::lesson_count).sum()
    }

    #[must_use]
    pub fn total_duration(&self) -> u32 {
        self.modules.iter().map(Module::total_duration).sum()
    }

    /// Return all lesson IDs in module order then lesson order.
    #[must_use]
    pub fn all_lesson_ids(&self) -> Vec<LessonId> {
        self.modules
            .iter()
            .flat_map(|m| m.lessons.iter().map(|l| l.id))
            .collect()
    }
}
