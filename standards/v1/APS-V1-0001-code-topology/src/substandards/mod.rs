//! Substandard implementations as feature-gated modules (ADR-0002).
//!
//! Each cargo feature name equals the substandard profile code (the id suffix
//! after the dot). The codes are cryptic on purpose, so here is the mapping to
//! each substandard's human name (from its `substandard.toml`):
//!
//! - `CI01` -> GitHub Actions CI Integration
//! - `MM01` -> Mermaid Diagram Projector
//! - `FD01` -> 3D Force-Directed Coupling Visualization
//! - `RS01` -> Rust Language Adapter
//! - `VZ01` -> Topology Visualization Dashboard

#[cfg(feature = "CI01")]
pub mod ci_github_actions;

#[cfg(feature = "MM01")]
pub mod viz_mermaid;

#[cfg(feature = "FD01")]
pub mod viz_3d;

#[cfg(feature = "RS01")]
pub mod lang_rust;

#[cfg(feature = "VZ01")]
pub mod viz_dashboard;
