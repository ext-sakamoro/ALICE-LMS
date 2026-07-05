//! system.

use crate::certificate::*;
use crate::course::Course;
use crate::enrollment::*;
use crate::grading::*;
use crate::ids::*;
use crate::progress::*;
use crate::quiz::*;
use std::collections::HashMap;

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
