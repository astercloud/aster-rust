// Agent Monitoring Module
//
// This module provides monitoring and observability:
// - Agent metrics collection and tracking
// - Alert management for threshold violations
// - Performance analysis and optimization suggestions

mod metrics;
mod alerts;
mod analyzer;

#[cfg(test)]
mod metrics_property_tests;

#[cfg(test)]
mod alerts_property_tests;

#[cfg(test)]
mod analyzer_property_tests;

pub use metrics::*;
pub use alerts::*;
pub use analyzer::*;
