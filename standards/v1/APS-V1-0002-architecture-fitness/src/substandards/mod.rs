//! Substandard implementations as feature-gated modules (ADR-0002).
//!
//! Each cargo feature name equals the substandard profile code (the id suffix
//! after the dot). The codes are cryptic on purpose, so here is the mapping to
//! each substandard's human name (from its `substandard.toml`):
//!
//! - `ST01` -> Structural Integrity Dimension
//! - `AC01` -> Accessibility Dimension
//! - `AV01` -> Availability Dimension
//! - `LG01` -> Legality Dimension
//! - `MD01` -> Modularity and Coupling Dimension
//! - `MT01` -> Maintainability Dimension
//! - `PF01` -> Performance Dimension
//! - `SC01` -> Security Dimension

#[cfg(feature = "ST01")]
pub mod structural;

#[cfg(feature = "AC01")]
pub mod accessibility;

#[cfg(feature = "AV01")]
pub mod availability;

#[cfg(feature = "LG01")]
pub mod legality;

#[cfg(feature = "MD01")]
pub mod modularity;

#[cfg(feature = "MT01")]
pub mod maintainability;

#[cfg(feature = "PF01")]
pub mod performance;

#[cfg(feature = "SC01")]
pub mod security;
