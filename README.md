**English** | [日本語](README_JP.md)

# ALICE-LMS

**ALICE Learning Management System** — Course management, progress tracking, quiz engine, grading, and certificate issuance.

Part of [Project A.L.I.C.E.](https://github.com/anthropics/alice) ecosystem.

## Features

- **Course Structure** — Hierarchical organization with Courses, Modules, and Lessons
- **Progress Tracking** — Per-student enrollment and lesson completion tracking
- **Quiz Engine** — Multiple-choice quiz creation, submission, and auto-grading
- **Grading System** — Configurable passing thresholds and grade calculation
- **Certificates** — Automatic certificate generation upon course completion
- **Duration Tracking** — Per-lesson and per-module duration aggregation

## Architecture

```
Course
 └── Module (ordered)
      └── Lesson (ordered, with duration)

Student
 ├── Enrollment → Course
 ├── Progress → Lesson completions
 └── Certificate (on completion)

QuizEngine
 ├── Quiz → Questions
 └── Submission → Auto-graded results
```

## Quick Start

```rust
use alice_lms::{Course, Module, Lesson, Lms};

let mut lms = Lms::new();
let mut course = Course::new(1, "Rust Fundamentals");
let mut module = Module::new(1, "Basics", 1);
module.add_lesson(Lesson::new(1, "Hello World", 1, 30));
course.add_module(module);
lms.add_course(course);
```

## License

MIT OR Apache-2.0
