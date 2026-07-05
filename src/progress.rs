//! progress.

use crate::ids::*;

// Progress tracking
// ---------------------------------------------------------------------------

/// Tracks a student's progress through a single course.
#[derive(Debug, Clone)]
pub struct Progress {
    pub student_id: StudentId,
    pub course_id: CourseId,
    completed_lessons: Vec<LessonId>,
    time_spent_minutes: u32,
    total_lessons: usize,
}

impl Progress {
    #[must_use]
    pub const fn new(student_id: StudentId, course_id: CourseId, total_lessons: usize) -> Self {
        Self {
            student_id,
            course_id,
            completed_lessons: Vec::new(),
            time_spent_minutes: 0,
            total_lessons,
        }
    }

    /// Mark a lesson as complete (idempotent).
    pub fn complete_lesson(&mut self, lesson_id: LessonId) {
        if !self.completed_lessons.contains(&lesson_id) {
            self.completed_lessons.push(lesson_id);
        }
    }

    /// Add time spent studying.
    pub const fn add_time(&mut self, minutes: u32) {
        self.time_spent_minutes += minutes;
    }

    /// Completion percentage (0..=100).
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn completion_percent(&self) -> u32 {
        if self.total_lessons == 0 {
            return 0;
        }
        let pct = (self.completed_lessons.len() * 100) / self.total_lessons;
        // pct is at most 100 (or slightly above with stale data), so truncation is safe.
        if pct > 100 {
            100
        } else {
            pct as u32
        }
    }

    #[must_use]
    pub const fn time_spent(&self) -> u32 {
        self.time_spent_minutes
    }

    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.completed_lessons.len() >= self.total_lessons
    }

    #[must_use]
    pub const fn completed_count(&self) -> usize {
        self.completed_lessons.len()
    }
}
