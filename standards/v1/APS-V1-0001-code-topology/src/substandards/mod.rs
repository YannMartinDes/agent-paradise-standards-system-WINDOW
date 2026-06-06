//! Substandard implementations as feature-gated modules (ADR-0002).

#[cfg(feature = "ci-github-actions")]
pub mod ci_github_actions;

#[cfg(feature = "viz-mermaid")]
pub mod viz_mermaid;

#[cfg(feature = "viz-3d")]
pub mod viz_3d;
