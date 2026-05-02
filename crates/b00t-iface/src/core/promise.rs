//! Promise — typed event that will be produced at a future lifecycle point.
//!
//! A promise is the fundamental unit of event-driven execution in the b00t surface
//! model. Every lifecycle transition (init → operate → maintain → terminate)
//! is a promise that resolves to a typed value or rejects with an error.
//!
//! # Event-driven operational capability
//!
//! Promises form a DAG: `InitPromise → OperatePromise → [MaintainPromise]* → TerminatePromise`.
//! The executive subsystem (see `exec::`) chains these promises and enforces
//! governance constraints between transitions.

use std::fmt::Debug;
use std::time::Duration;

/// The resolved value of a lifecycle promise.
#[derive(Debug, Clone)]
pub enum PromiseValue<T, E> {
    Fulfilled(T),
    Rejected(E),
}

/// A lifecycle promise that carries a typed outcome and metadata.
#[derive(Debug, Clone)]
pub struct LifecyclePromise<T, E> {
    /// The operation this promise represents.
    pub operation: PromiseOp,
    /// Duration the operation took.
    pub elapsed: Duration,
    /// The outcome.
    pub value: PromiseValue<T, E>,
}

/// Operations that produce promises.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromiseOp {
    Init,
    Operate,
    Maintain,
    Terminate,
}

impl std::fmt::Display for PromiseOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Init => write!(f, "init"),
            Self::Operate => write!(f, "operate"),
            Self::Maintain => write!(f, "maintain"),
            Self::Terminate => write!(f, "terminate"),
        }
    }
}

impl<T, E> LifecyclePromise<T, E> {
    pub fn new(op: PromiseOp, elapsed: Duration, value: PromiseValue<T, E>) -> Self {
        Self { operation: op, elapsed, value }
    }

    pub fn fulfilled(op: PromiseOp, elapsed: Duration, value: T) -> Self {
        Self::new(op, elapsed, PromiseValue::Fulfilled(value))
    }

    pub fn rejected(op: PromiseOp, elapsed: Duration, error: E) -> Self {
        Self::new(op, elapsed, PromiseValue::Rejected(error))
    }

    pub fn is_fulfilled(&self) -> bool {
        matches!(self.value, PromiseValue::Fulfilled(_))
    }

    pub fn is_rejected(&self) -> bool {
        matches!(self.value, PromiseValue::Rejected(_))
    }
}

/// A chain of promises forming a complete lifecycle. Errors are always `String`
/// for solver-readable audit trails.
#[derive(Debug, Clone)]
pub struct PromiseChain<H: Clone> {
    pub init: LifecyclePromise<(), String>,
    pub operate: LifecyclePromise<H, String>,
    pub maintains: Vec<LifecyclePromise<super::surface::MaintenanceAction, String>>,
    pub terminate: LifecyclePromise<super::surface::AuditRecord, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promise_fulfilled() {
        let p = LifecyclePromise::<u32, String>::fulfilled(PromiseOp::Init, Duration::from_secs(1), 42);
        assert!(p.is_fulfilled());
        assert!(!p.is_rejected());
        assert_eq!(p.operation.to_string(), "init");
    }

    #[test]
    fn promise_rejected() {
        let p = LifecyclePromise::<u32, String>::rejected(PromiseOp::Operate, Duration::from_secs(2), "kaboom".into());
        assert!(p.is_rejected());
        assert!(!p.is_fulfilled());
    }

    #[test]
    fn promise_op_display() {
        assert_eq!(PromiseOp::Terminate.to_string(), "terminate");
        assert_eq!(PromiseOp::Maintain.to_string(), "maintain");
    }
}
