// Agent Communication Module
//
// This module provides inter-agent communication capabilities:
// - Message bus for agent-to-agent messaging
// - Shared state management
// - Agent coordination and task assignment

mod message_bus;
mod shared_state;
mod coordinator;

#[cfg(test)]
mod message_bus_property_tests;
#[cfg(test)]
mod shared_state_property_tests;
#[cfg(test)]
mod coordinator_property_tests;

pub use message_bus::*;
pub use shared_state::*;
pub use coordinator::*;
