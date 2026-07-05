//! certificate.

use crate::ids::*;

// Certificate
// ---------------------------------------------------------------------------

/// Criteria required before a certificate can be issued.
#[derive(Debug, Clone)]
pub struct CertCriteria {
    pub min_completion_percent: u32,
    pub min_quiz_pass_count: u32,
    pub min_gpa: f64,
}

impl CertCriteria {
    #[must_use]
    pub const fn new(min_completion_percent: u32, min_quiz_pass_count: u32, min_gpa: f64) -> Self {
        Self {
            min_completion_percent,
            min_quiz_pass_count,
            min_gpa,
        }
    }

    /// Check whether the student meets the criteria.
    #[must_use]
    pub fn is_met(&self, completion_percent: u32, quiz_pass_count: u32, gpa: f64) -> bool {
        completion_percent >= self.min_completion_percent
            && quiz_pass_count >= self.min_quiz_pass_count
            && gpa >= self.min_gpa
    }
}

/// An issued certificate.
#[derive(Debug, Clone)]
pub struct Certificate {
    pub id: CertificateId,
    pub student_id: StudentId,
    pub course_id: CourseId,
    pub student_name: String,
    pub course_title: String,
    pub gpa: f64,
    pub issued_date: String,
}

impl Certificate {
    #[must_use]
    pub fn new(
        id: CertificateId,
        student_id: StudentId,
        course_id: CourseId,
        student_name: &str,
        course_title: &str,
        gpa: f64,
        issued_date: &str,
    ) -> Self {
        Self {
            id,
            student_id,
            course_id,
            student_name: student_name.to_owned(),
            course_title: course_title.to_owned(),
            gpa,
            issued_date: issued_date.to_owned(),
        }
    }
}
