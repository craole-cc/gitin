use crate::{PromptResult, PromptWithOptions};

pub fn permission_granted<S>(message: S) -> bool
where
    S: AsRef<str>,
{
    let prompt_options = PromptWithOptions {
        message: message.as_ref(),
        options: vec![("y", "Yes"), ("n", "No")],
        default: ("n", "No"),
    };

    match prompt_options.prompt() {
        PromptResult::Success(selection) => !prompt_options.is_default(&selection),
        PromptResult::Error(error) => {
            eprintln!("{}", error);
            false
        }
    }
}

#[macro_export]
macro_rules! permission_granted {
    ($message:expr) => {
        permission_granted($message)
    };
}

#[test]
fn test_permission_granted_yes() {
    assert!(permission_granted("Continue?"));
}

#[test]
fn test_permission_granted_no() {
    assert!(!permission_granted("Continue?"));
}
