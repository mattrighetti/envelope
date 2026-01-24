use std::io::Result;

use zeroize::Zeroizing;

use crate::std_err;

/// Prompts for password input.
///
/// Returns the password wrapped in `Zeroizing` to ensure it is securely erased
/// from memory when dropped
pub(crate) fn prompt_password(prompt: &str) -> Result<Zeroizing<String>> {
    rpassword::prompt_password(prompt)
        .map(Zeroizing::new)
        .map_err(|e| std_err!("failed to read password: {}", e))
}

/// Prompts for password with confirmation, returns error if they don't match.
///
/// Returns the password wrapped in `Zeroizing` to ensure it is securely erased
/// from memory when dropped
pub(crate) fn prompt_password_confirm() -> Result<Zeroizing<String>> {
    let password = prompt_password("Password: ")?;

    if password.is_empty() {
        return Err(std_err!("password cannot be empty"));
    }

    let confirm = prompt_password("Confirm password: ")?;

    if *password != *confirm {
        return Err(std_err!("passwords do not match"));
    }

    Ok(password)
}
