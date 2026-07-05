//! grading.

use crate::ids::*;

// Grading
// ---------------------------------------------------------------------------

/// A single graded item with a weight.
#[derive(Debug, Clone)]
pub struct GradeEntry {
    pub label: String,
    pub score: f64,
    pub max: f64,
    pub weight: f64,
}

impl GradeEntry {
    #[must_use]
    pub fn new(label: &str, score: f64, max: f64, weight: f64) -> Self {
        Self {
            label: label.to_owned(),
            score,
            max,
            weight,
        }
    }

    #[must_use]
    pub fn ratio(&self) -> f64 {
        if self.max == 0.0 {
            0.0
        } else {
            self.score / self.max
        }
    }
}

/// Grade book for one student in one course.
#[derive(Debug, Clone)]
pub struct GradeBook {
    pub student_id: StudentId,
    pub course_id: CourseId,
    pub(crate) entries: Vec<GradeEntry>,
}

impl GradeBook {
    #[must_use]
    pub const fn new(student_id: StudentId, course_id: CourseId) -> Self {
        Self {
            student_id,
            course_id,
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: GradeEntry) {
        self.entries.push(entry);
    }

    /// Weighted average (0.0..=1.0).
    #[must_use]
    pub fn weighted_average(&self) -> f64 {
        let total_weight: f64 = self.entries.iter().map(|e| e.weight).sum();
        if total_weight == 0.0 {
            return 0.0;
        }
        let weighted_sum: f64 = self.entries.iter().map(|e| e.ratio() * e.weight).sum();
        weighted_sum / total_weight
    }

    /// Convert weighted average to a 4.0 GPA scale.
    #[must_use]
    pub fn gpa(&self) -> f64 {
        let avg = self.weighted_average();
        gpa_from_ratio(avg)
    }

    /// Letter grade from weighted average.
    #[must_use]
    pub fn letter_grade(&self) -> &'static str {
        letter_from_ratio(self.weighted_average())
    }

    #[must_use]
    pub const fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

/// Map a ratio (0.0..=1.0) to a 4.0-scale GPA.
#[must_use]
pub fn gpa_from_ratio(ratio: f64) -> f64 {
    if ratio >= 0.9 {
        4.0
    } else if ratio >= 0.8 {
        3.0
    } else if ratio >= 0.7 {
        2.0
    } else if ratio >= 0.6 {
        1.0
    } else {
        0.0
    }
}

/// Map a ratio to a letter grade.
#[must_use]
pub fn letter_from_ratio(ratio: f64) -> &'static str {
    if ratio >= 0.9 {
        "A"
    } else if ratio >= 0.8 {
        "B"
    } else if ratio >= 0.7 {
        "C"
    } else if ratio >= 0.6 {
        "D"
    } else {
        "F"
    }
}
