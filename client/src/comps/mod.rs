//! This module just holds components.

#![allow(non_camel_case_types)]

pub mod error_message;
pub mod list_of_tests_and_completions;
pub mod login_form;
pub mod navbar;
pub mod test_and_completions;

pub use self::{
    error_message::ErrorMessage, list_of_tests_and_completions::ListOfTestsAndCompletions,
    login_form::LoginOrCreateAccountForm, navbar::Navbar, test_and_completions::TestAndCompletions,
};
