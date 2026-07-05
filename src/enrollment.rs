//! enrollment.

use crate::ids::*;

// Enrollment
// ---------------------------------------------------------------------------

/// Status of a student's enrollment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnrollmentStatus {
    Active,
    Completed,
    Dropped,
    Pending,
}

/// An enrollment record.
#[derive(Debug, Clone)]
pub struct Enrollment {
    pub student_id: StudentId,
    pub course_id: CourseId,
    pub status: EnrollmentStatus,
    pub enrolled_date: String,
}

impl Enrollment {
    #[must_use]
    pub fn new(student_id: StudentId, course_id: CourseId, enrolled_date: &str) -> Self {
        Self {
            student_id,
            course_id,
            status: EnrollmentStatus::Pending,
            enrolled_date: enrolled_date.to_owned(),
        }
    }

    pub const fn activate(&mut self) {
        self.status = EnrollmentStatus::Active;
    }

    pub const fn complete(&mut self) {
        self.status = EnrollmentStatus::Completed;
    }

    pub const fn drop_enrollment(&mut self) {
        self.status = EnrollmentStatus::Dropped;
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.status == EnrollmentStatus::Active
    }
}
