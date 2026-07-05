//! quiz.

use crate::ids::*;

// Quiz engine
// ---------------------------------------------------------------------------

/// The kind of question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionKind {
    /// Multiple-choice with index of correct answer (0-based).
    MultipleChoice {
        choices: Vec<String>,
        correct: usize,
    },
    /// Fill-in-the-blank with expected answer (case-insensitive match).
    FillIn { expected: String },
}

/// A single question.
#[derive(Debug, Clone)]
pub struct Question {
    pub text: String,
    pub kind: QuestionKind,
    pub points: u32,
}

impl Question {
    #[must_use]
    pub fn multiple_choice(text: &str, choices: &[&str], correct: usize, points: u32) -> Self {
        Self {
            text: text.to_owned(),
            kind: QuestionKind::MultipleChoice {
                choices: choices.iter().map(|&s| s.to_owned()).collect(),
                correct,
            },
            points,
        }
    }

    #[must_use]
    pub fn fill_in(text: &str, expected: &str, points: u32) -> Self {
        Self {
            text: text.to_owned(),
            kind: QuestionKind::FillIn {
                expected: expected.to_owned(),
            },
            points,
        }
    }

    /// Check whether `answer` is correct.
    #[must_use]
    pub fn check(&self, answer: &Answer) -> bool {
        match (&self.kind, answer) {
            (QuestionKind::MultipleChoice { correct, .. }, Answer::Choice(idx)) => *idx == *correct,
            (QuestionKind::FillIn { expected }, Answer::Text(txt)) => {
                txt.eq_ignore_ascii_case(expected)
            }
            _ => false,
        }
    }
}

/// An answer submitted by a student.
#[derive(Debug, Clone)]
pub enum Answer {
    Choice(usize),
    Text(String),
}

/// A quiz attached to a course.
#[derive(Debug, Clone)]
pub struct Quiz {
    pub id: QuizId,
    pub course_id: CourseId,
    pub title: String,
    pub questions: Vec<Question>,
    pub passing_score: u32,
}

impl Quiz {
    #[must_use]
    pub fn new(id: QuizId, course_id: CourseId, title: &str, passing_score: u32) -> Self {
        Self {
            id,
            course_id,
            title: title.to_owned(),
            questions: Vec::new(),
            passing_score,
        }
    }

    pub fn add_question(&mut self, question: Question) {
        self.questions.push(question);
    }

    #[must_use]
    pub fn max_score(&self) -> u32 {
        self.questions.iter().map(|q| q.points).sum()
    }

    /// Grade a set of answers. Returns `(earned, max, passed)`.
    #[must_use]
    pub fn grade(&self, answers: &[Answer]) -> QuizResult {
        let mut earned: u32 = 0;
        for (q, a) in self.questions.iter().zip(answers.iter()) {
            if q.check(a) {
                earned += q.points;
            }
        }
        let max = self.max_score();
        let passed = earned >= self.passing_score;
        QuizResult {
            earned,
            max,
            passed,
        }
    }
}

/// Result of grading a quiz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuizResult {
    pub earned: u32,
    pub max: u32,
    pub passed: bool,
}

impl QuizResult {
    /// Percentage score (0..=100).
    #[must_use]
    pub const fn percentage(&self) -> u32 {
        if self.max == 0 {
            return 0;
        }
        (self.earned * 100) / self.max
    }
}
