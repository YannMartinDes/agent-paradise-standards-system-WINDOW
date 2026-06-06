//! Substandard implementations as feature-gated modules (ADR-0002).

#[cfg(feature = "ci-github-actions")]
pub mod ci_github_actions;
