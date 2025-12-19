//! Stubs for External Dependencies (Event Framework, Quant Lib).
//!
//! This module provides stub implementations for external systems that are
//! out of scope for this library:
//! - QuantLibStub: Mock implementation of the Quant Library for testing
//! - EventFrameworkStub: Mock implementation of the Event Framework for orchestration
//! - Thin Slice Handlers: Demo implementations for continuous barrier products
//!
//! These stubs enable end-to-end testing of the two-call pattern without
//! requiring actual external system integration.
//!
//! **Note**: This module is for testing and demonstration purposes only.
//! Production implementations would integrate with real external systems.

pub mod event_framework;
pub mod quant_lib;
pub mod thin_slice;

// Re-export main types for convenience
pub use event_framework::{
    EventFramework, EventFrameworkError, EventFrameworkResult, EventFrameworkStub,
    SimulationResult, TriggerRegistration,
};

pub use quant_lib::{QuantLibStub, QuantLibTrait};

pub use thin_slice::{
    ApplyResultProto, KnockOutWarrantHandler, MiniCertificateHandler, QuantLibStub as ThinSliceQuantLibStub,
    TwoCallFlowResult, create_thin_slice_rules, execute_two_call_flow,
};
