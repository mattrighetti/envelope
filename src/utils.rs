use anyhow::{Context, Result, ensure};
use zeroize::Zeroizing;

/// Prompts for password input.
///
/// Returns the password wrapped in `Zeroizing` to ensure it is securely erased
/// from memory when dropped
pub(crate) fn prompt_password(prompt: &str) -> Result<Zeroizing<String>> {
    rpassword::prompt_password(prompt)
        .map(Zeroizing::new)
        .context("failed to read password")
}

/// Prompts for password with confirmation, returns error if they don't match.
///
/// Returns the password wrapped in `Zeroizing` to ensure it is securely erased
/// from memory when dropped
pub(crate) fn prompt_password_confirm() -> Result<Zeroizing<String>> {
    let password = prompt_password("Password: ")?;

    ensure!(!password.is_empty(), "password cannot be empty");

    let confirm = prompt_password("Confirm password: ")?;

    ensure!(*password == *confirm, "passwords do not match");

    Ok(password)
}
