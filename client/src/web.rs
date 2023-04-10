//! This module handles various interfaces to web APIs.

use crate::STORAGE_KEY_USER;
use gloo_utils::window;
use serde::Deserialize;
use test_tracker_shared::User;
use web_sys::Storage;

/// Return the `localStorage`.
pub fn local_storage() -> Storage {
    /// Avoid repitition with the .expect() calls.
    const EXPECT_STRING: &str =
        "The client should only be run in web environments, so localStorage should always exist";
    window()
        .local_storage()
        .expect(EXPECT_STRING)
        .expect(EXPECT_STRING)
}

/// Return the `sessionStorage`.
pub fn session_storage() -> Storage {
    /// Avoid repitition with the .expect() calls.
    const EXPECT_STRING: &str =
        "The client should only be run in web environments, so sessionStorage should always exist";
    window()
        .session_storage()
        .expect(EXPECT_STRING)
        .expect(EXPECT_STRING)
}

/// Get an item from the given storage.
fn get_item_from_storage<T: for<'a> Deserialize<'a>>(storage: Storage, key: &str) -> Option<T> {
    storage
        .get_item(key)
        .map(|value| value.map(|s| ron::from_str::<T>(&s).ok()))
        .ok()
        .flatten()
        .flatten()
}

/// Try `localStorage`, then `sessionStorage` for the user.
pub fn get_user() -> Option<User> {
    if let Some(user) = get_item_from_storage(local_storage(), STORAGE_KEY_USER) {
        Some(user)
    } else {
        get_item_from_storage(session_storage(), STORAGE_KEY_USER)
    }
}
