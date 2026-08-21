//! Tests for the traits that the errors of the crate implement
//!
//! A caller sends the errors of this crate between threads and keeps them in
//! a report that another thread reads. These tests hold the crate to the
//! auto traits that make this possible, because a private field of a later
//! version could take them away without a word from the compiler.

use kawauso_config::error::DeserializeConfigurationError;
use kawauso_config::error::LoadConfigurationError;

#[test]
fn deserialize_configuration_error_is_send_and_sync() {
    fn assert_send_and_sync<T: Send + Sync>() {}

    assert_send_and_sync::<DeserializeConfigurationError>();
}

#[test]
fn load_configuration_error_is_send_and_sync() {
    fn assert_send_and_sync<T: Send + Sync>() {}

    assert_send_and_sync::<LoadConfigurationError>();
}
