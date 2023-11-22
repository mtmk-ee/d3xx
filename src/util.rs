//! Internal utility types and functions.

use std::marker::PhantomData;

/// Used to force a lifetime constraint on a type which does not contain any references.
///
/// This is important for ensuring that lifetimes are correctly enforced by the type
/// system, which otherwise would not catch use-after-free errors in this crate.
pub type PhantomLifetime<'a, T = ()> = PhantomData<&'a T>;

/// Used to force a type to be `!Sync`.
pub type PhantomUnsync = PhantomData<std::cell::Cell<()>>;
