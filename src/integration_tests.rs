//! Integration tests.

#![allow(
    clippy::wildcard_imports,
    clippy::too_many_lines,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use crate::certificate::*;
use crate::course::*;
use crate::enrollment::*;
use crate::grading::*;
use crate::ids::*;
use crate::lesson::*;
use crate::module::*;
use crate::progress::*;
use crate::quiz::*;
use crate::system::*;
use std::collections::HashMap;

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
    assert!((cert.gpa - 4.0).abs() < f64::EPSILON);
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
        m.add_lesson(Lesson::new(
            i,
            &format!("L{i}"),
            u32::try_from(i).unwrap(),
            5,
        ));
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
