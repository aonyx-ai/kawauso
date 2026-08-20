//! Tests for the traits that the error of the crate implements
//!
//! A caller sends the error of this crate between threads and keeps it in a
//! report that another thread reads. These tests hold the crate to the auto
//! traits that make this possible, because a private field of a later version
//! could take them away without a word from the compiler.

use kawauso_config::DeserializeConfigurationError;

#[test]
fn deserialize_configuration_error_is_send_and_sync() {
    fn assert_send_and_sync<T: Send + Sync>() {}

    assert_send_and_sync::<DeserializeConfigurationError>();
}
