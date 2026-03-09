#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

//! ALICE-LMS: Learning Management System
//!
//! Course management, progress tracking, quiz engine, grading,
//! certificates, and enrollment management.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// ID types
// ---------------------------------------------------------------------------

/// Unique identifier for a course.
pub type CourseId = u64;
/// Unique identifier for a module within a course.
pub type ModuleId = u64;
/// Unique identifier for a lesson within a module.
pub type LessonId = u64;
/// Unique identifier for a quiz.
pub type QuizId = u64;
/// Unique identifier for a student.
pub type StudentId = u64;
/// Unique identifier for a certificate.
pub type CertificateId = u64;

// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
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
    entries: Vec<GradeEntry>,
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

// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
// LMS — top-level system
// ---------------------------------------------------------------------------

/// The top-level Learning Management System.
#[derive(Debug, Default)]
pub struct Lms {
    courses: HashMap<CourseId, Course>,
    quizzes: HashMap<QuizId, Quiz>,
    enrollments: Vec<Enrollment>,
    progress: HashMap<(StudentId, CourseId), Progress>,
    grade_books: HashMap<(StudentId, CourseId), GradeBook>,
    certificates: Vec<Certificate>,
    next_cert_id: CertificateId,
}

impl Lms {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // -- Course management --------------------------------------------------

    pub fn add_course(&mut self, course: Course) {
        self.courses.insert(course.id, course);
    }

    #[must_use]
    pub fn get_course(&self, id: CourseId) -> Option<&Course> {
        self.courses.get(&id)
    }

    pub fn get_course_mut(&mut self, id: CourseId) -> Option<&mut Course> {
        self.courses.get_mut(&id)
    }

    #[must_use]
    pub fn course_count(&self) -> usize {
        self.courses.len()
    }

    // -- Quiz management ----------------------------------------------------

    pub fn add_quiz(&mut self, quiz: Quiz) {
        self.quizzes.insert(quiz.id, quiz);
    }

    #[must_use]
    pub fn get_quiz(&self, id: QuizId) -> Option<&Quiz> {
        self.quizzes.get(&id)
    }

    pub fn get_quiz_mut(&mut self, id: QuizId) -> Option<&mut Quiz> {
        self.quizzes.get_mut(&id)
    }

    #[must_use]
    pub fn quiz_count(&self) -> usize {
        self.quizzes.len()
    }

    // -- Enrollment ---------------------------------------------------------

    pub fn enroll(&mut self, student_id: StudentId, course_id: CourseId, date: &str) {
        let mut enrollment = Enrollment::new(student_id, course_id, date);
        enrollment.activate();
        self.enrollments.push(enrollment);

        let total = self
            .courses
            .get(&course_id)
            .map_or(0, Course::total_lessons);
        self.progress.insert(
            (student_id, course_id),
            Progress::new(student_id, course_id, total),
        );
        self.grade_books.insert(
            (student_id, course_id),
            GradeBook::new(student_id, course_id),
        );
    }

    #[must_use]
    pub fn is_enrolled(&self, student_id: StudentId, course_id: CourseId) -> bool {
        self.enrollments
            .iter()
            .any(|e| e.student_id == student_id && e.course_id == course_id && e.is_active())
    }

    pub fn drop_student(&mut self, student_id: StudentId, course_id: CourseId) {
        for e in &mut self.enrollments {
            if e.student_id == student_id && e.course_id == course_id {
                e.drop_enrollment();
            }
        }
    }

    #[must_use]
    pub const fn enrollment_count(&self) -> usize {
        self.enrollments.len()
    }

    #[must_use]
    pub fn active_enrollments(&self) -> usize {
        self.enrollments.iter().filter(|e| e.is_active()).count()
    }

    #[must_use]
    pub fn enrollments_for_student(&self, student_id: StudentId) -> Vec<&Enrollment> {
        self.enrollments
            .iter()
            .filter(|e| e.student_id == student_id)
            .collect()
    }

    #[must_use]
    pub fn enrollments_for_course(&self, course_id: CourseId) -> Vec<&Enrollment> {
        self.enrollments
            .iter()
            .filter(|e| e.course_id == course_id)
            .collect()
    }

    // -- Progress -----------------------------------------------------------

    pub fn complete_lesson(
        &mut self,
        student_id: StudentId,
        course_id: CourseId,
        lesson_id: LessonId,
    ) {
        if let Some(p) = self.progress.get_mut(&(student_id, course_id)) {
            p.complete_lesson(lesson_id);
        }
    }

    pub fn add_time(&mut self, student_id: StudentId, course_id: CourseId, minutes: u32) {
        if let Some(p) = self.progress.get_mut(&(student_id, course_id)) {
            p.add_time(minutes);
        }
    }

    #[must_use]
    pub fn completion_percent(&self, student_id: StudentId, course_id: CourseId) -> u32 {
        self.progress
            .get(&(student_id, course_id))
            .map_or(0, Progress::completion_percent)
    }

    #[must_use]
    pub fn time_spent(&self, student_id: StudentId, course_id: CourseId) -> u32 {
        self.progress
            .get(&(student_id, course_id))
            .map_or(0, Progress::time_spent)
    }

    #[must_use]
    pub fn get_progress(&self, student_id: StudentId, course_id: CourseId) -> Option<&Progress> {
        self.progress.get(&(student_id, course_id))
    }

    // -- Quiz submission & grading ------------------------------------------

    /// Submit answers for a quiz, record the grade, and return the result.
    pub fn submit_quiz(
        &mut self,
        student_id: StudentId,
        quiz_id: QuizId,
        answers: &[Answer],
    ) -> Option<QuizResult> {
        let quiz = self.quizzes.get(&quiz_id)?;
        let result = quiz.grade(answers);
        let course_id = quiz.course_id;
        let title = quiz.title.clone();
        let max = f64::from(result.max);
        let earned = f64::from(result.earned);

        if let Some(gb) = self.grade_books.get_mut(&(student_id, course_id)) {
            gb.add_entry(GradeEntry::new(&title, earned, max, 1.0));
        }

        Some(result)
    }

    #[must_use]
    pub fn get_grade_book(&self, student_id: StudentId, course_id: CourseId) -> Option<&GradeBook> {
        self.grade_books.get(&(student_id, course_id))
    }

    pub fn get_grade_book_mut(
        &mut self,
        student_id: StudentId,
        course_id: CourseId,
    ) -> Option<&mut GradeBook> {
        self.grade_books.get_mut(&(student_id, course_id))
    }

    // -- Certificates -------------------------------------------------------

    /// Try to issue a certificate. Returns the certificate if criteria are met.
    pub fn issue_certificate(
        &mut self,
        student_id: StudentId,
        course_id: CourseId,
        student_name: &str,
        criteria: &CertCriteria,
        date: &str,
    ) -> Option<Certificate> {
        let completion = self.completion_percent(student_id, course_id);
        let gpa = self
            .grade_books
            .get(&(student_id, course_id))
            .map_or(0.0, GradeBook::gpa);

        // Count passed quizzes for this course.
        let quiz_pass_count: u32 = self
            .grade_books
            .get(&(student_id, course_id))
            .map_or(0, |gb| {
                // Each entry with ratio >= passing threshold counts.
                // We use 0.6 as minimum pass ratio (matching GPA D = 1.0).
                u32::try_from(gb.entries.iter().filter(|e| e.ratio() >= 0.6).count())
                    .unwrap_or(u32::MAX)
            });

        let course_title = self
            .courses
            .get(&course_id)
            .map_or_else(String::new, |c| c.title.clone());

        if criteria.is_met(completion, quiz_pass_count, gpa) {
            let cert = Certificate::new(
                self.next_cert_id,
                student_id,
                course_id,
                student_name,
                &course_title,
                gpa,
                date,
            );
            self.next_cert_id += 1;
            self.certificates.push(cert.clone());
            Some(cert)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn certificate_count(&self) -> usize {
        self.certificates.len()
    }

    #[must_use]
    pub fn certificates_for_student(&self, student_id: StudentId) -> Vec<&Certificate> {
        self.certificates
            .iter()
            .filter(|c| c.student_id == student_id)
            .collect()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- Lesson tests -------------------------------------------------------

    #[test]
    fn lesson_new() {
        let l = Lesson::new(1, "Intro", 0, 30);
        assert_eq!(l.id, 1);
        assert_eq!(l.title, "Intro");
        assert_eq!(l.order, 0);
        assert_eq!(l.duration_minutes, 30);
    }

    #[test]
    fn lesson_clone() {
        let l = Lesson::new(1, "A", 0, 10);
        let l2 = l.clone();
        assert_eq!(l2.id, l.id);
    }

    // -- Module tests -------------------------------------------------------

    #[test]
    fn module_new_empty() {
        let m = Module::new(1, "Basics", 0);
        assert_eq!(m.lesson_count(), 0);
        assert_eq!(m.total_duration(), 0);
    }

    #[test]
    fn module_add_lesson() {
        let mut m = Module::new(1, "Basics", 0);
        m.add_lesson(Lesson::new(1, "L1", 1, 10));
        m.add_lesson(Lesson::new(2, "L2", 0, 20));
        assert_eq!(m.lesson_count(), 2);
        // Sorted by order: L2(0) comes first
        assert_eq!(m.lessons[0].id, 2);
    }

    #[test]
    fn module_total_duration() {
        let mut m = Module::new(1, "M", 0);
        m.add_lesson(Lesson::new(1, "A", 0, 15));
        m.add_lesson(Lesson::new(2, "B", 1, 25));
        assert_eq!(m.total_duration(), 40);
    }

    // -- Course tests -------------------------------------------------------

    fn make_course() -> Course {
        let mut c = Course::new(1, "Rust 101", "Learn Rust");
        let mut m1 = Module::new(1, "Basics", 0);
        m1.add_lesson(Lesson::new(1, "Hello World", 0, 10));
        m1.add_lesson(Lesson::new(2, "Variables", 1, 15));
        let mut m2 = Module::new(2, "Advanced", 1);
        m2.add_lesson(Lesson::new(3, "Lifetimes", 0, 20));
        c.add_module(m1);
        c.add_module(m2);
        c
    }

    #[test]
    fn course_new() {
        let c = Course::new(1, "T", "D");
        assert_eq!(c.id, 1);
        assert_eq!(c.total_lessons(), 0);
    }

    #[test]
    fn course_total_lessons() {
        let c = make_course();
        assert_eq!(c.total_lessons(), 3);
    }

    #[test]
    fn course_total_duration() {
        let c = make_course();
        assert_eq!(c.total_duration(), 45);
    }

    #[test]
    fn course_all_lesson_ids() {
        let c = make_course();
        assert_eq!(c.all_lesson_ids(), vec![1, 2, 3]);
    }

    #[test]
    fn course_module_ordering() {
        let mut c = Course::new(1, "C", "D");
        c.add_module(Module::new(2, "Second", 2));
        c.add_module(Module::new(1, "First", 1));
        assert_eq!(c.modules[0].id, 1);
        assert_eq!(c.modules[1].id, 2);
    }

    // -- Question / Quiz tests ----------------------------------------------

    #[test]
    fn multiple_choice_correct() {
        let q = Question::multiple_choice("Q?", &["A", "B", "C"], 1, 10);
        assert!(q.check(&Answer::Choice(1)));
    }

    #[test]
    fn multiple_choice_wrong() {
        let q = Question::multiple_choice("Q?", &["A", "B", "C"], 1, 10);
        assert!(!q.check(&Answer::Choice(0)));
    }

    #[test]
    fn fill_in_correct() {
        let q = Question::fill_in("Capital of Japan?", "Tokyo", 5);
        assert!(q.check(&Answer::Text("tokyo".to_owned())));
    }

    #[test]
    fn fill_in_case_insensitive() {
        let q = Question::fill_in("Q?", "Rust", 5);
        assert!(q.check(&Answer::Text("RUST".to_owned())));
    }

    #[test]
    fn fill_in_wrong() {
        let q = Question::fill_in("Q?", "Rust", 5);
        assert!(!q.check(&Answer::Text("Python".to_owned())));
    }

    #[test]
    fn wrong_answer_type() {
        let q = Question::fill_in("Q?", "Rust", 5);
        assert!(!q.check(&Answer::Choice(0)));
    }

    #[test]
    fn quiz_max_score() {
        let mut quiz = Quiz::new(1, 1, "Final", 60);
        quiz.add_question(Question::multiple_choice("Q1", &["A", "B"], 0, 50));
        quiz.add_question(Question::fill_in("Q2", "yes", 50));
        assert_eq!(quiz.max_score(), 100);
    }

    #[test]
    fn quiz_grade_all_correct() {
        let mut quiz = Quiz::new(1, 1, "Quiz", 60);
        quiz.add_question(Question::multiple_choice("Q1", &["A", "B"], 0, 50));
        quiz.add_question(Question::fill_in("Q2", "yes", 50));
        let result = quiz.grade(&[Answer::Choice(0), Answer::Text("yes".to_owned())]);
        assert_eq!(result.earned, 100);
        assert!(result.passed);
    }

    #[test]
    fn quiz_grade_partial() {
        let mut quiz = Quiz::new(1, 1, "Quiz", 60);
        quiz.add_question(Question::multiple_choice("Q1", &["A", "B"], 0, 50));
        quiz.add_question(Question::fill_in("Q2", "yes", 50));
        let result = quiz.grade(&[Answer::Choice(0), Answer::Text("no".to_owned())]);
        assert_eq!(result.earned, 50);
        assert!(!result.passed);
    }

    #[test]
    fn quiz_grade_none_correct() {
        let mut quiz = Quiz::new(1, 1, "Quiz", 60);
        quiz.add_question(Question::multiple_choice("Q1", &["A", "B"], 0, 50));
        let result = quiz.grade(&[Answer::Choice(1)]);
        assert_eq!(result.earned, 0);
        assert!(!result.passed);
    }

    #[test]
    fn quiz_grade_empty() {
        let quiz = Quiz::new(1, 1, "Empty", 0);
        let result = quiz.grade(&[]);
        assert_eq!(result.earned, 0);
        assert!(result.passed); // 0 >= 0
    }

    #[test]
    fn quiz_result_percentage() {
        let r = QuizResult {
            earned: 75,
            max: 100,
            passed: true,
        };
        assert_eq!(r.percentage(), 75);
    }

    #[test]
    fn quiz_result_percentage_zero_max() {
        let r = QuizResult {
            earned: 0,
            max: 0,
            passed: true,
        };
        assert_eq!(r.percentage(), 0);
    }

    // -- Progress tests -----------------------------------------------------

    #[test]
    fn progress_new() {
        let p = Progress::new(1, 1, 5);
        assert_eq!(p.completion_percent(), 0);
        assert!(!p.is_complete());
    }

    #[test]
    fn progress_complete_lessons() {
        let mut p = Progress::new(1, 1, 4);
        p.complete_lesson(1);
        p.complete_lesson(2);
        assert_eq!(p.completion_percent(), 50);
        assert_eq!(p.completed_count(), 2);
    }

    #[test]
    fn progress_complete_all() {
        let mut p = Progress::new(1, 1, 2);
        p.complete_lesson(1);
        p.complete_lesson(2);
        assert_eq!(p.completion_percent(), 100);
        assert!(p.is_complete());
    }

    #[test]
    fn progress_idempotent_completion() {
        let mut p = Progress::new(1, 1, 2);
        p.complete_lesson(1);
        p.complete_lesson(1);
        assert_eq!(p.completed_count(), 1);
    }

    #[test]
    fn progress_time_tracking() {
        let mut p = Progress::new(1, 1, 2);
        p.add_time(30);
        p.add_time(15);
        assert_eq!(p.time_spent(), 45);
    }

    #[test]
    fn progress_zero_total() {
        let p = Progress::new(1, 1, 0);
        assert_eq!(p.completion_percent(), 0);
    }

    // -- Grade tests --------------------------------------------------------

    #[test]
    fn grade_entry_ratio() {
        let e = GradeEntry::new("Quiz 1", 80.0, 100.0, 1.0);
        assert!((e.ratio() - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn grade_entry_ratio_zero_max() {
        let e = GradeEntry::new("X", 0.0, 0.0, 1.0);
        assert!((e.ratio()).abs() < f64::EPSILON);
    }

    #[test]
    fn grade_book_empty() {
        let gb = GradeBook::new(1, 1);
        assert!((gb.weighted_average()).abs() < f64::EPSILON);
        assert!((gb.gpa()).abs() < f64::EPSILON);
    }

    #[test]
    fn grade_book_single_entry() {
        let mut gb = GradeBook::new(1, 1);
        gb.add_entry(GradeEntry::new("Q1", 90.0, 100.0, 1.0));
        assert!((gb.weighted_average() - 0.9).abs() < f64::EPSILON);
        assert!((gb.gpa() - 4.0).abs() < f64::EPSILON);
        assert_eq!(gb.letter_grade(), "A");
    }

    #[test]
    fn grade_book_weighted_average() {
        let mut gb = GradeBook::new(1, 1);
        gb.add_entry(GradeEntry::new("Q1", 100.0, 100.0, 0.3));
        gb.add_entry(GradeEntry::new("Q2", 50.0, 100.0, 0.7));
        // (1.0 * 0.3 + 0.5 * 0.7) / 1.0 = 0.65
        let avg = gb.weighted_average();
        assert!((avg - 0.65).abs() < 1e-10);
    }

    #[test]
    fn grade_book_entry_count() {
        let mut gb = GradeBook::new(1, 1);
        gb.add_entry(GradeEntry::new("A", 1.0, 1.0, 1.0));
        gb.add_entry(GradeEntry::new("B", 1.0, 1.0, 1.0));
        assert_eq!(gb.entry_count(), 2);
    }

    #[test]
    fn gpa_a() {
        assert!((gpa_from_ratio(0.95) - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gpa_b() {
        assert!((gpa_from_ratio(0.85) - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gpa_c() {
        assert!((gpa_from_ratio(0.75) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gpa_d() {
        assert!((gpa_from_ratio(0.65) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gpa_f() {
        assert!((gpa_from_ratio(0.5)).abs() < f64::EPSILON);
    }

    #[test]
    fn letter_a() {
        assert_eq!(letter_from_ratio(0.9), "A");
    }

    #[test]
    fn letter_b() {
        assert_eq!(letter_from_ratio(0.8), "B");
    }

    #[test]
    fn letter_c() {
        assert_eq!(letter_from_ratio(0.7), "C");
    }

    #[test]
    fn letter_d() {
        assert_eq!(letter_from_ratio(0.6), "D");
    }

    #[test]
    fn letter_f() {
        assert_eq!(letter_from_ratio(0.5), "F");
    }

    // -- Certificate criteria tests -----------------------------------------

    #[test]
    fn cert_criteria_met() {
        let c = CertCriteria::new(100, 1, 3.0);
        assert!(c.is_met(100, 2, 4.0));
    }

    #[test]
    fn cert_criteria_not_met_completion() {
        let c = CertCriteria::new(100, 1, 3.0);
        assert!(!c.is_met(80, 2, 4.0));
    }

    #[test]
    fn cert_criteria_not_met_quiz() {
        let c = CertCriteria::new(100, 2, 3.0);
        assert!(!c.is_met(100, 1, 4.0));
    }

    #[test]
    fn cert_criteria_not_met_gpa() {
        let c = CertCriteria::new(100, 1, 3.0);
        assert!(!c.is_met(100, 1, 2.0));
    }

    #[test]
    fn certificate_new() {
        let cert = Certificate::new(0, 1, 1, "Alice", "Rust 101", 4.0, "2026-03-09");
        assert_eq!(cert.student_name, "Alice");
        assert_eq!(cert.gpa, 4.0);
    }

    // -- Enrollment tests ---------------------------------------------------

    #[test]
    fn enrollment_new_is_pending() {
        let e = Enrollment::new(1, 1, "2026-01-01");
        assert_eq!(e.status, EnrollmentStatus::Pending);
    }

    #[test]
    fn enrollment_activate() {
        let mut e = Enrollment::new(1, 1, "2026-01-01");
        e.activate();
        assert!(e.is_active());
    }

    #[test]
    fn enrollment_complete() {
        let mut e = Enrollment::new(1, 1, "2026-01-01");
        e.activate();
        e.complete();
        assert_eq!(e.status, EnrollmentStatus::Completed);
        assert!(!e.is_active());
    }

    #[test]
    fn enrollment_drop() {
        let mut e = Enrollment::new(1, 1, "2026-01-01");
        e.activate();
        e.drop_enrollment();
        assert_eq!(e.status, EnrollmentStatus::Dropped);
    }

    // -- LMS integration tests ----------------------------------------------

    fn setup_lms() -> Lms {
        let mut lms = Lms::new();

        let mut course = Course::new(1, "Rust 101", "Learn Rust");
        let mut m = Module::new(1, "Basics", 0);
        m.add_lesson(Lesson::new(1, "Hello", 0, 10));
        m.add_lesson(Lesson::new(2, "Types", 1, 15));
        m.add_lesson(Lesson::new(3, "Functions", 2, 20));
        course.add_module(m);
        lms.add_course(course);

        let mut quiz = Quiz::new(1, 1, "Midterm", 60);
        quiz.add_question(Question::multiple_choice("Q1", &["A", "B", "C"], 0, 50));
        quiz.add_question(Question::fill_in("Q2", "rust", 50));
        lms.add_quiz(quiz);

        lms
    }

    #[test]
    fn lms_add_course() {
        let lms = setup_lms();
        assert_eq!(lms.course_count(), 1);
    }

    #[test]
    fn lms_get_course() {
        let lms = setup_lms();
        let c = lms.get_course(1).unwrap();
        assert_eq!(c.title, "Rust 101");
    }

    #[test]
    fn lms_get_course_none() {
        let lms = setup_lms();
        assert!(lms.get_course(999).is_none());
    }

    #[test]
    fn lms_get_course_mut() {
        let mut lms = setup_lms();
        let c = lms.get_course_mut(1).unwrap();
        c.title = "Rust 201".to_owned();
        assert_eq!(lms.get_course(1).unwrap().title, "Rust 201");
    }

    #[test]
    fn lms_add_quiz() {
        let lms = setup_lms();
        assert_eq!(lms.quiz_count(), 1);
    }

    #[test]
    fn lms_get_quiz() {
        let lms = setup_lms();
        assert!(lms.get_quiz(1).is_some());
    }

    #[test]
    fn lms_get_quiz_none() {
        let lms = setup_lms();
        assert!(lms.get_quiz(999).is_none());
    }

    #[test]
    fn lms_enroll_student() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        assert!(lms.is_enrolled(100, 1));
        assert_eq!(lms.enrollment_count(), 1);
        assert_eq!(lms.active_enrollments(), 1);
    }

    #[test]
    fn lms_not_enrolled() {
        let lms = setup_lms();
        assert!(!lms.is_enrolled(100, 1));
    }

    #[test]
    fn lms_drop_student() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        lms.drop_student(100, 1);
        assert!(!lms.is_enrolled(100, 1));
        assert_eq!(lms.active_enrollments(), 0);
    }

    #[test]
    fn lms_enrollments_for_student() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        assert_eq!(lms.enrollments_for_student(100).len(), 1);
    }

    #[test]
    fn lms_enrollments_for_course() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        lms.enroll(101, 1, "2026-01-02");
        assert_eq!(lms.enrollments_for_course(1).len(), 2);
    }

    #[test]
    fn lms_complete_lesson_progress() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        lms.complete_lesson(100, 1, 1);
        assert_eq!(lms.completion_percent(100, 1), 33); // 1/3
    }

    #[test]
    fn lms_complete_all_lessons() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        lms.complete_lesson(100, 1, 1);
        lms.complete_lesson(100, 1, 2);
        lms.complete_lesson(100, 1, 3);
        assert_eq!(lms.completion_percent(100, 1), 100);
    }

    #[test]
    fn lms_time_tracking() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        lms.add_time(100, 1, 30);
        lms.add_time(100, 1, 15);
        assert_eq!(lms.time_spent(100, 1), 45);
    }

    #[test]
    fn lms_time_not_enrolled() {
        let lms = setup_lms();
        assert_eq!(lms.time_spent(100, 1), 0);
    }

    #[test]
    fn lms_get_progress() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        assert!(lms.get_progress(100, 1).is_some());
    }

    #[test]
    fn lms_get_progress_none() {
        let lms = setup_lms();
        assert!(lms.get_progress(100, 1).is_none());
    }

    #[test]
    fn lms_submit_quiz_pass() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        let result = lms
            .submit_quiz(
                100,
                1,
                &[Answer::Choice(0), Answer::Text("rust".to_owned())],
            )
            .unwrap();
        assert_eq!(result.earned, 100);
        assert!(result.passed);
    }

    #[test]
    fn lms_submit_quiz_fail() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        let result = lms
            .submit_quiz(100, 1, &[Answer::Choice(2), Answer::Text("go".to_owned())])
            .unwrap();
        assert_eq!(result.earned, 0);
        assert!(!result.passed);
    }

    #[test]
    fn lms_submit_quiz_nonexistent() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        assert!(lms.submit_quiz(100, 999, &[]).is_none());
    }

    #[test]
    fn lms_grade_book_after_quiz() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        lms.submit_quiz(
            100,
            1,
            &[Answer::Choice(0), Answer::Text("rust".to_owned())],
        );
        let gb = lms.get_grade_book(100, 1).unwrap();
        assert_eq!(gb.entry_count(), 1);
        assert!((gb.weighted_average() - 1.0).abs() < f64::EPSILON);
        assert!((gb.gpa() - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn lms_grade_book_none() {
        let lms = setup_lms();
        assert!(lms.get_grade_book(100, 1).is_none());
    }

    #[test]
    fn lms_grade_book_mut() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        let gb = lms.get_grade_book_mut(100, 1).unwrap();
        gb.add_entry(GradeEntry::new("HW", 50.0, 100.0, 1.0));
        assert_eq!(lms.get_grade_book(100, 1).unwrap().entry_count(), 1);
    }

    #[test]
    fn lms_issue_certificate_success() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");

        // Complete all lessons
        lms.complete_lesson(100, 1, 1);
        lms.complete_lesson(100, 1, 2);
        lms.complete_lesson(100, 1, 3);

        // Pass quiz (score 100/100 -> ratio 1.0 -> GPA 4.0)
        lms.submit_quiz(
            100,
            1,
            &[Answer::Choice(0), Answer::Text("rust".to_owned())],
        );

        let criteria = CertCriteria::new(100, 1, 4.0);
        let cert = lms
            .issue_certificate(100, 1, "Alice", &criteria, "2026-03-09")
            .unwrap();
        assert_eq!(cert.student_name, "Alice");
        assert_eq!(cert.course_title, "Rust 101");
        assert_eq!(lms.certificate_count(), 1);
    }

    #[test]
    fn lms_issue_certificate_fail_completion() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        lms.submit_quiz(
            100,
            1,
            &[Answer::Choice(0), Answer::Text("rust".to_owned())],
        );

        let criteria = CertCriteria::new(100, 1, 4.0);
        assert!(lms
            .issue_certificate(100, 1, "Alice", &criteria, "2026-03-09")
            .is_none());
    }

    #[test]
    fn lms_issue_certificate_fail_gpa() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        lms.complete_lesson(100, 1, 1);
        lms.complete_lesson(100, 1, 2);
        lms.complete_lesson(100, 1, 3);
        // Fail quiz -> GPA 0
        lms.submit_quiz(100, 1, &[Answer::Choice(2), Answer::Text("go".to_owned())]);

        let criteria = CertCriteria::new(100, 1, 3.0);
        assert!(lms
            .issue_certificate(100, 1, "Alice", &criteria, "2026-03-09")
            .is_none());
    }

    #[test]
    fn lms_certificates_for_student() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        lms.complete_lesson(100, 1, 1);
        lms.complete_lesson(100, 1, 2);
        lms.complete_lesson(100, 1, 3);
        lms.submit_quiz(
            100,
            1,
            &[Answer::Choice(0), Answer::Text("rust".to_owned())],
        );
        let criteria = CertCriteria::new(100, 1, 4.0);
        lms.issue_certificate(100, 1, "Alice", &criteria, "2026-03-09");
        assert_eq!(lms.certificates_for_student(100).len(), 1);
        assert_eq!(lms.certificates_for_student(999).len(), 0);
    }

    #[test]
    fn lms_certificate_id_increments() {
        let mut lms = setup_lms();

        // Second course
        let mut c2 = Course::new(2, "Go 101", "Learn Go");
        let mut m = Module::new(10, "Intro", 0);
        m.add_lesson(Lesson::new(10, "Basics", 0, 5));
        c2.add_module(m);
        lms.add_course(c2);

        let mut q2 = Quiz::new(2, 2, "Go Quiz", 0);
        q2.add_question(Question::fill_in("X?", "x", 10));
        lms.add_quiz(q2);

        // Student 100 completes course 1
        lms.enroll(100, 1, "2026-01-01");
        for lid in 1..=3 {
            lms.complete_lesson(100, 1, lid);
        }
        lms.submit_quiz(
            100,
            1,
            &[Answer::Choice(0), Answer::Text("rust".to_owned())],
        );

        // Student 100 completes course 2
        lms.enroll(100, 2, "2026-01-01");
        lms.complete_lesson(100, 2, 10);
        lms.submit_quiz(100, 2, &[Answer::Text("x".to_owned())]);

        let crit = CertCriteria::new(100, 1, 4.0);
        let c1 = lms
            .issue_certificate(100, 1, "Alice", &crit, "2026-03-09")
            .unwrap();
        let c2 = lms
            .issue_certificate(100, 2, "Alice", &crit, "2026-03-09")
            .unwrap();
        assert_eq!(c1.id, 0);
        assert_eq!(c2.id, 1);
    }

    #[test]
    fn lms_default() {
        let lms = Lms::default();
        assert_eq!(lms.course_count(), 0);
        assert_eq!(lms.quiz_count(), 0);
        assert_eq!(lms.enrollment_count(), 0);
        assert_eq!(lms.certificate_count(), 0);
    }

    #[test]
    fn lms_multiple_students() {
        let mut lms = setup_lms();
        lms.enroll(100, 1, "2026-01-01");
        lms.enroll(101, 1, "2026-01-02");
        lms.complete_lesson(100, 1, 1);
        assert_eq!(lms.completion_percent(100, 1), 33);
        assert_eq!(lms.completion_percent(101, 1), 0);
    }

    #[test]
    fn lms_get_quiz_mut() {
        let mut lms = setup_lms();
        let q = lms.get_quiz_mut(1).unwrap();
        q.passing_score = 80;
        assert_eq!(lms.get_quiz(1).unwrap().passing_score, 80);
    }

    // -- Edge case tests ----------------------------------------------------

    #[test]
    fn quiz_fewer_answers_than_questions() {
        let mut quiz = Quiz::new(1, 1, "Q", 0);
        quiz.add_question(Question::fill_in("A?", "a", 10));
        quiz.add_question(Question::fill_in("B?", "b", 10));
        // Only answer first question
        let result = quiz.grade(&[Answer::Text("a".to_owned())]);
        assert_eq!(result.earned, 10);
    }

    #[test]
    fn course_empty_modules() {
        let c = Course::new(1, "Empty", "No modules");
        assert_eq!(c.total_lessons(), 0);
        assert_eq!(c.total_duration(), 0);
        assert!(c.all_lesson_ids().is_empty());
    }

    #[test]
    fn module_with_many_lessons() {
        let mut m = Module::new(1, "Big", 0);
        for i in 0..50 {
            m.add_lesson(Lesson::new(i, &format!("L{i}"), i as u32, 5));
        }
        assert_eq!(m.lesson_count(), 50);
        assert_eq!(m.total_duration(), 250);
    }

    #[test]
    fn grade_book_multiple_weights() {
        let mut gb = GradeBook::new(1, 1);
        gb.add_entry(GradeEntry::new("HW", 100.0, 100.0, 0.2));
        gb.add_entry(GradeEntry::new("Mid", 80.0, 100.0, 0.3));
        gb.add_entry(GradeEntry::new("Final", 90.0, 100.0, 0.5));
        // (1.0*0.2 + 0.8*0.3 + 0.9*0.5) / 1.0 = 0.2 + 0.24 + 0.45 = 0.89
        let avg = gb.weighted_average();
        assert!((avg - 0.89).abs() < 1e-10);
        assert!((gb.gpa() - 3.0).abs() < f64::EPSILON); // 0.89 -> B -> 3.0
        assert_eq!(gb.letter_grade(), "B");
    }

    #[test]
    fn progress_completion_capped() {
        // Edge: more completions than total (stale data scenario)
        let mut p = Progress::new(1, 1, 1);
        p.complete_lesson(1);
        p.complete_lesson(2); // extra
        assert_eq!(p.completion_percent(), 100);
    }

    #[test]
    fn enrollment_date_preserved() {
        let e = Enrollment::new(1, 1, "2026-06-15");
        assert_eq!(e.enrolled_date, "2026-06-15");
    }

    #[test]
    fn lms_completion_not_enrolled() {
        let lms = setup_lms();
        assert_eq!(lms.completion_percent(999, 1), 0);
    }

    #[test]
    fn cert_criteria_all_zero() {
        let c = CertCriteria::new(0, 0, 0.0);
        assert!(c.is_met(0, 0, 0.0));
    }

    #[test]
    fn quiz_single_question_pass() {
        let mut quiz = Quiz::new(1, 1, "One", 5);
        quiz.add_question(Question::fill_in("1+1?", "2", 10));
        let r = quiz.grade(&[Answer::Text("2".to_owned())]);
        assert!(r.passed);
        assert_eq!(r.percentage(), 100);
    }

    #[test]
    fn quiz_single_question_fail() {
        let mut quiz = Quiz::new(1, 1, "One", 5);
        quiz.add_question(Question::fill_in("1+1?", "2", 10));
        let r = quiz.grade(&[Answer::Text("3".to_owned())]);
        assert!(!r.passed);
        assert_eq!(r.percentage(), 0);
    }

    #[test]
    fn multiple_choice_type_mismatch() {
        let q = Question::multiple_choice("Q?", &["A"], 0, 10);
        assert!(!q.check(&Answer::Text("A".to_owned())));
    }

    #[test]
    fn fill_in_empty_answer() {
        let q = Question::fill_in("Q?", "answer", 5);
        assert!(!q.check(&Answer::Text(String::new())));
    }

    #[test]
    fn lms_multiple_courses() {
        let mut lms = Lms::new();
        lms.add_course(Course::new(1, "A", ""));
        lms.add_course(Course::new(2, "B", ""));
        lms.add_course(Course::new(3, "C", ""));
        assert_eq!(lms.course_count(), 3);
    }

    #[test]
    fn lms_multiple_quizzes() {
        let mut lms = Lms::new();
        lms.add_quiz(Quiz::new(1, 1, "Q1", 10));
        lms.add_quiz(Quiz::new(2, 1, "Q2", 20));
        assert_eq!(lms.quiz_count(), 2);
    }
}
