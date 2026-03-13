[English](README.md) | **日本語**

# ALICE-LMS

**ALICE 学習管理システム** — コース管理、進捗追跡、クイズエンジン、成績評価、修了証発行。

[Project A.L.I.C.E.](https://github.com/anthropics/alice) エコシステムの一部。

## 機能

- **コース構造** — コース・モジュール・レッスンの階層的管理
- **進捗追跡** — 受講者ごとの登録状況・レッスン完了の追跡
- **クイズエンジン** — 選択式クイズの作成・提出・自動採点
- **成績評価** — 合格基準の設定と成績計算
- **修了証** — コース完了時の自動証明書発行
- **時間管理** — レッスン・モジュール単位の所要時間集計

## アーキテクチャ

```
Course（コース）
 └── Module（モジュール、順序付き）
      └── Lesson（レッスン、順序・所要時間付き）

Student（受講者）
 ├── Enrollment → コースへの登録
 ├── Progress → レッスン完了記録
 └── Certificate（修了時に発行）

QuizEngine（クイズエンジン）
 ├── Quiz → 問題群
 └── Submission → 自動採点結果
```

## クイックスタート

```rust
use alice_lms::{Course, Module, Lesson, Lms};

let mut lms = Lms::new();
let mut course = Course::new(1, "Rust基礎");
let mut module = Module::new(1, "基本", 1);
module.add_lesson(Lesson::new(1, "Hello World", 1, 30));
course.add_module(module);
lms.add_course(course);
```

## ライセンス

MIT OR Apache-2.0
