//! `signed_certificate` — Ed25519-signed course-completion certificates
//! and enrollment audit trail.
//!
//! Every learner event (enrolment, module completion, quiz attempt,
//! certificate issue, certificate revocation) is captured in a signed
//! record chained via `prev_hash → hash`. Employers verifying a
//! candidate's certificate can retrieve the record, re-hash it, and
//! confirm the issuing institution's signature — without contacting the
//! institution.
//!
//! # Regulatory alignment
//!
//! - **`ISO 9001` §7.2** — competence records must be retained; signed
//!   certificates are the definitive evidence of achieved competence.
//! - **`SCORM 2004` 4th Edition** — completion status (`cmi.completion_status`)
//!   integrity for tracked learning; the chain hardens the LMS record
//!   against silent tampering.
//! - **`GDPR` Art. 20** — right to data portability; a portable signed
//!   certificate lets the learner take their credentials to another
//!   platform.
//! - **`FERPA` (20 U.S.C. §1232g)** — education records must be
//!   protected from unauthorised disclosure and modification.
//! - **`W3C Verifiable Credentials Data Model 2.0`** — the record layout
//!   maps directly to `VC.issuer`, `VC.credentialSubject`, and `VC.proof`.
//!
//! Cryptographic primitives are provided by `alice-blockchain` (`Ed25519`).

#![allow(
    clippy::doc_markdown,
    clippy::missing_panics_doc,
    clippy::too_many_arguments,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation
)]

use alice_blockchain::signature::{KeyPair, PublicKey, Signature};

// ---------------------------------------------------------------------------
// LearningEventKind
// ---------------------------------------------------------------------------

/// The learning event captured in the trail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LearningEventKind {
    /// A learner enrolled in a course.
    Enrolled,
    /// A learning module was completed.
    ModuleCompleted,
    /// A quiz or assessment attempt was recorded.
    QuizAttempt,
    /// A certificate was issued to the learner.
    CertificateIssued,
    /// A previously issued certificate was revoked.
    CertificateRevoked,
    /// The learner withdrew from the course.
    Withdrawn,
}

impl LearningEventKind {
    /// Short code used in canonical serialization.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Enrolled => "ENROL",
            Self::ModuleCompleted => "MOD",
            Self::QuizAttempt => "QUIZ",
            Self::CertificateIssued => "CERT",
            Self::CertificateRevoked => "REVOKE",
            Self::Withdrawn => "WDR",
        }
    }
}

// ---------------------------------------------------------------------------
// LearningRecord
// ---------------------------------------------------------------------------

/// One learning event ready to be signed.
///
/// Score is expressed as basis points 0..=10_000 (100.00 % = 10_000) to
/// avoid floating-point representation issues in signed payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningRecord {
    /// Monotonic sequence number.
    pub seq: u64,
    /// Kind of event.
    pub kind: LearningEventKind,
    /// Unix nanosecond timestamp.
    pub timestamp_ns: u64,
    /// Learner identifier (student id, email, DID).
    pub learner_id: String,
    /// Course identifier.
    pub course_id: String,
    /// Optional module / assessment identifier (empty when not applicable).
    pub module_id: String,
    /// Score in basis points (0..=10_000), or 0 when not applicable.
    pub score_bps: u32,
    /// Certificate identifier (empty unless kind is
    /// [`LearningEventKind::CertificateIssued`] or
    /// [`LearningEventKind::CertificateRevoked`]).
    pub certificate_id: String,
    /// Free-form detail (issuing institution, remarks).
    pub detail: String,
    /// Hash of the previous record (0 for genesis).
    pub prev_hash: u64,
}

impl LearningRecord {
    /// Canonical byte layout used for hashing and signing.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(200);
        buf.extend_from_slice(&self.seq.to_le_bytes());
        buf.extend_from_slice(self.kind.code().as_bytes());
        buf.push(0);
        buf.extend_from_slice(&self.timestamp_ns.to_le_bytes());
        buf.extend_from_slice(self.learner_id.as_bytes());
        buf.push(0);
        buf.extend_from_slice(self.course_id.as_bytes());
        buf.push(0);
        buf.extend_from_slice(self.module_id.as_bytes());
        buf.push(0);
        buf.extend_from_slice(&self.score_bps.to_le_bytes());
        buf.extend_from_slice(self.certificate_id.as_bytes());
        buf.push(0);
        buf.extend_from_slice(self.detail.as_bytes());
        buf.push(0);
        buf.extend_from_slice(&self.prev_hash.to_le_bytes());
        buf
    }

    /// `FNV-1a` hash of the canonical byte layout.
    #[must_use]
    pub fn hash(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for &b in &self.canonical_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }
}

// ---------------------------------------------------------------------------
// SignedLearningRecord
// ---------------------------------------------------------------------------

/// [`LearningRecord`] plus the issuing institution's `Ed25519` signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedLearningRecord {
    /// The wrapped record.
    pub record: LearningRecord,
    /// `FNV-1a` hash of the record's canonical bytes.
    pub hash: u64,
    /// `Ed25519` signature over the canonical bytes.
    pub signature: Signature,
    /// Institution's `Ed25519` public key.
    pub issuer: PublicKey,
}

impl SignedLearningRecord {
    /// Verify signature and hash consistency.
    #[must_use]
    pub fn verify(&self) -> bool {
        if self.hash != self.record.hash() {
            return false;
        }
        self.issuer
            .verify(&self.record.canonical_bytes(), &self.signature)
    }
}

// ---------------------------------------------------------------------------
// CertificateTrail
// ---------------------------------------------------------------------------

/// Append-only chain of [`SignedLearningRecord`] records.
#[derive(Debug, Clone, Default)]
pub struct CertificateTrail {
    entries: Vec<SignedLearningRecord>,
}

impl CertificateTrail {
    /// Construct an empty trail.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Number of entries.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the trail is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Read-only view.
    #[must_use]
    pub fn entries(&self) -> &[SignedLearningRecord] {
        &self.entries
    }

    /// Hash of the last record (0 for empty).
    #[must_use]
    pub fn tail_hash(&self) -> u64 {
        self.entries.last().map_or(0, |e| e.hash)
    }

    /// Append a new learning event signed with the issuer's key pair.
    pub fn append(
        &mut self,
        keypair: &KeyPair,
        kind: LearningEventKind,
        timestamp_ns: u64,
        learner_id: impl Into<String>,
        course_id: impl Into<String>,
        module_id: impl Into<String>,
        score_bps: u32,
        certificate_id: impl Into<String>,
        detail: impl Into<String>,
    ) -> &SignedLearningRecord {
        let seq = self.entries.len() as u64;
        let prev_hash = self.tail_hash();
        let record = LearningRecord {
            seq,
            kind,
            timestamp_ns,
            learner_id: learner_id.into(),
            course_id: course_id.into(),
            module_id: module_id.into(),
            score_bps,
            certificate_id: certificate_id.into(),
            detail: detail.into(),
            prev_hash,
        };
        let bytes = record.canonical_bytes();
        let hash = record.hash();
        let signature = keypair.sign(&bytes);
        let issuer = keypair.public();
        self.entries.push(SignedLearningRecord {
            record,
            hash,
            signature,
            issuer,
        });
        self.entries.last().expect("entry was just pushed")
    }

    /// Verify signature and chain integrity end-to-end.
    #[must_use]
    pub fn find_first_tamper(&self) -> Option<usize> {
        let mut expected_prev: u64 = 0;
        for (i, e) in self.entries.iter().enumerate() {
            if e.record.seq as usize != i {
                return Some(i);
            }
            if e.record.prev_hash != expected_prev {
                return Some(i);
            }
            if !e.verify() {
                return Some(i);
            }
            expected_prev = e.hash;
        }
        None
    }

    /// Whether the trail is intact.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.find_first_tamper().is_none()
    }

    /// Whether the given certificate id has been issued **and** not
    /// subsequently revoked.
    #[must_use]
    pub fn is_certificate_valid(&self, certificate_id: &str) -> bool {
        let mut issued = false;
        for e in &self.entries {
            if e.record.certificate_id != certificate_id {
                continue;
            }
            match e.record.kind {
                LearningEventKind::CertificateIssued => issued = true,
                LearningEventKind::CertificateRevoked => issued = false,
                _ => {}
            }
        }
        issued
    }

    /// All distinct learner ids seen in the trail.
    #[must_use]
    pub fn learners(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for e in &self.entries {
            if !out.contains(&e.record.learner_id) {
                out.push(e.record.learner_id.clone());
            }
        }
        out
    }

    /// Count events of the given kind on the given course.
    #[must_use]
    pub fn count_kind_for_course(&self, course_id: &str, kind: LearningEventKind) -> usize {
        self.entries
            .iter()
            .filter(|e| e.record.course_id == course_id && e.record.kind == kind)
            .count()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn kp(seed: u8) -> KeyPair {
        KeyPair::from_seed([seed; 32])
    }

    #[test]
    fn kind_code_is_stable() {
        assert_eq!(LearningEventKind::Enrolled.code(), "ENROL");
        assert_eq!(LearningEventKind::ModuleCompleted.code(), "MOD");
        assert_eq!(LearningEventKind::QuizAttempt.code(), "QUIZ");
        assert_eq!(LearningEventKind::CertificateIssued.code(), "CERT");
        assert_eq!(LearningEventKind::CertificateRevoked.code(), "REVOKE");
        assert_eq!(LearningEventKind::Withdrawn.code(), "WDR");
    }

    #[test]
    fn canonical_bytes_are_deterministic() {
        let r = LearningRecord {
            seq: 0,
            kind: LearningEventKind::CertificateIssued,
            timestamp_ns: 1,
            learner_id: String::from("L-001"),
            course_id: String::from("C-INTRO"),
            module_id: String::new(),
            score_bps: 8500,
            certificate_id: String::from("CERT-1"),
            detail: String::from("Introduction to ALICE"),
            prev_hash: 0,
        };
        assert_eq!(r.canonical_bytes(), r.canonical_bytes());
    }

    #[test]
    fn hash_differs_when_score_changes() {
        let mut r = LearningRecord {
            seq: 0,
            kind: LearningEventKind::QuizAttempt,
            timestamp_ns: 1,
            learner_id: String::from("L-001"),
            course_id: String::from("C-1"),
            module_id: String::from("Q-1"),
            score_bps: 8000,
            certificate_id: String::new(),
            detail: String::new(),
            prev_hash: 0,
        };
        let h1 = r.hash();
        r.score_bps = 10000;
        assert_ne!(h1, r.hash());
    }

    #[test]
    fn empty_trail_tail_hash_is_zero() {
        let trail = CertificateTrail::new();
        assert_eq!(trail.tail_hash(), 0);
        assert!(trail.is_empty());
    }

    #[test]
    fn signed_record_verifies_on_append() {
        let mut trail = CertificateTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            LearningEventKind::Enrolled,
            1,
            "L-001",
            "C-INTRO",
            "",
            0,
            "",
            "",
        );
        assert!(trail.entries()[0].verify());
    }

    #[test]
    fn chained_prev_hash_matches_predecessor() {
        let mut trail = CertificateTrail::new();
        let k = kp(1);
        trail.append(&k, LearningEventKind::Enrolled, 1, "L", "C", "", 0, "", "");
        trail.append(
            &k,
            LearningEventKind::ModuleCompleted,
            2,
            "L",
            "C",
            "M-1",
            10000,
            "",
            "",
        );
        let first = trail.entries()[0].hash;
        assert_eq!(trail.entries()[1].record.prev_hash, first);
    }

    #[test]
    fn intact_completion_arc_is_valid() {
        let mut trail = CertificateTrail::new();
        let k = kp(1);
        trail.append(&k, LearningEventKind::Enrolled, 1, "L", "C", "", 0, "", "");
        trail.append(
            &k,
            LearningEventKind::ModuleCompleted,
            2,
            "L",
            "C",
            "M-1",
            10000,
            "",
            "",
        );
        trail.append(
            &k,
            LearningEventKind::QuizAttempt,
            3,
            "L",
            "C",
            "Q-1",
            8500,
            "",
            "",
        );
        trail.append(
            &k,
            LearningEventKind::CertificateIssued,
            4,
            "L",
            "C",
            "",
            8500,
            "CERT-1",
            "",
        );
        assert!(trail.is_valid());
    }

    #[test]
    fn tampered_score_is_detected() {
        let mut trail = CertificateTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            LearningEventKind::QuizAttempt,
            1,
            "L",
            "C",
            "Q-1",
            6000,
            "",
            "",
        );
        // Attacker inflates the score.
        trail.entries[0].record.score_bps = 10000;
        assert!(!trail.entries[0].verify());
        assert_eq!(trail.find_first_tamper(), Some(0));
    }

    #[test]
    fn tampered_certificate_id_is_detected() {
        let mut trail = CertificateTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            LearningEventKind::CertificateIssued,
            1,
            "L",
            "C",
            "",
            10000,
            "CERT-original",
            "",
        );
        trail.entries[0].record.certificate_id = String::from("CERT-attacker");
        assert!(!trail.entries[0].verify());
    }

    #[test]
    fn foreign_issuer_signature_is_rejected() {
        let mut trail = CertificateTrail::new();
        let genuine = kp(1);
        let attacker = kp(2);
        trail.append(
            &genuine,
            LearningEventKind::CertificateIssued,
            1,
            "L",
            "C",
            "",
            10000,
            "CERT-1",
            "",
        );
        let bytes = trail.entries[0].record.canonical_bytes();
        trail.entries[0].signature = attacker.sign(&bytes);
        assert!(!trail.entries[0].verify());
    }

    #[test]
    fn issued_then_revoked_certificate_is_invalid() {
        let mut trail = CertificateTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            LearningEventKind::CertificateIssued,
            1,
            "L",
            "C",
            "",
            10000,
            "CERT-1",
            "",
        );
        trail.append(
            &k,
            LearningEventKind::CertificateRevoked,
            2,
            "L",
            "C",
            "",
            0,
            "CERT-1",
            "found to be forged",
        );
        assert!(!trail.is_certificate_valid("CERT-1"));
    }

    #[test]
    fn issued_certificate_is_valid() {
        let mut trail = CertificateTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            LearningEventKind::CertificateIssued,
            1,
            "L",
            "C",
            "",
            10000,
            "CERT-1",
            "",
        );
        assert!(trail.is_certificate_valid("CERT-1"));
        assert!(!trail.is_certificate_valid("CERT-unknown"));
    }

    #[test]
    fn learners_lists_distinct() {
        let mut trail = CertificateTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            LearningEventKind::Enrolled,
            1,
            "L-A",
            "C",
            "",
            0,
            "",
            "",
        );
        trail.append(
            &k,
            LearningEventKind::Enrolled,
            2,
            "L-B",
            "C",
            "",
            0,
            "",
            "",
        );
        trail.append(
            &k,
            LearningEventKind::CertificateIssued,
            3,
            "L-A",
            "C",
            "",
            10000,
            "CERT-A",
            "",
        );
        let learners = trail.learners();
        assert_eq!(learners.len(), 2);
        assert!(learners.contains(&String::from("L-A")));
        assert!(learners.contains(&String::from("L-B")));
    }

    #[test]
    fn count_kind_for_course_filters() {
        let mut trail = CertificateTrail::new();
        let k = kp(1);
        for i in 0..3 {
            trail.append(
                &k,
                LearningEventKind::Enrolled,
                i,
                format!("L-{i}"),
                "C-1",
                "",
                0,
                "",
                "",
            );
        }
        for i in 0..2 {
            trail.append(
                &k,
                LearningEventKind::Enrolled,
                i,
                format!("L-x{i}"),
                "C-2",
                "",
                0,
                "",
                "",
            );
        }
        assert_eq!(
            trail.count_kind_for_course("C-1", LearningEventKind::Enrolled),
            3
        );
        assert_eq!(
            trail.count_kind_for_course("C-2", LearningEventKind::Enrolled),
            2
        );
        assert_eq!(
            trail.count_kind_for_course("C-1", LearningEventKind::CertificateIssued),
            0
        );
    }

    #[test]
    fn different_kinds_produce_different_hashes() {
        let mk = |kind: LearningEventKind| LearningRecord {
            seq: 0,
            kind,
            timestamp_ns: 1,
            learner_id: String::new(),
            course_id: String::new(),
            module_id: String::new(),
            score_bps: 0,
            certificate_id: String::new(),
            detail: String::new(),
            prev_hash: 0,
        };
        assert_ne!(
            mk(LearningEventKind::Enrolled).hash(),
            mk(LearningEventKind::CertificateIssued).hash()
        );
    }
}
