use std::io::{stdin, stdout, Write};

pub struct PromptWithOptions<'a> {
    pub message: &'a str,
    pub options: Vec<(&'a str, &'a str)>, // Change the type to Vec<(&'a str, &'a str)>
    pub default: (&'a str, &'a str),
}

pub enum PromptResult {
    Success(String),
    Error(String),
}

impl<'a> PromptWithOptions<'a> {
    pub fn with_message(mut self, message: &'a str) -> Self {
        self.message = message;
        self
    }

    pub fn with_option(mut self, key: &'a str, value: &'a str) -> Self {
        self.options.push((key, value)); // Push the new option to the vector
        self
    }

    pub fn with_default_key(mut self, default_key: &'a str) -> Self {
        self.default = (default_key, self.default.1);
        self
    }

    pub fn with_default_value(mut self, default_value: &'a str) -> Self {
        self.default = (self.default.0, default_value);
        self
    }

    fn get_option_value(&self, key: &str) -> Option<String> {
        self.options
            .iter()
            .find(|(option_key, _)| option_key.to_lowercase() == key.to_lowercase())
            .map(|(_, value)| value.to_string())
    }

    fn get_option_keys(&self) -> Vec<String> {
        self.options
            .iter()
            .map(|(option_key, _)| option_key.to_string())
            .collect()
    }

    fn default_option_key(&self) -> String {
        self.default.0.to_string()
    }

    fn print(&self) {
        //> Clone options to avoid borrowing issues during sorting
        let mut sorted_options: Vec<_> = self.options.to_vec();

        //> Sort the options first by key and then by value
        sorted_options.sort_by(|a, b| {
            let key_cmp = a.0.cmp(b.0);
            if key_cmp == std::cmp::Ordering::Equal {
                a.1.cmp(b.1)
            } else {
                key_cmp
            }
        });

        let mut formatted_options = String::new(); // create an empty String

        for (key, value) in sorted_options {
            let default_tag = if key == self.default.0 && value == self.default.1 {
                "[Default]"
            } else {
                ""
            };
            formatted_options.push_str(&format!("\t{}: {} {}\n", key, value, default_tag));
        }

        // Print the concatenated message
        print!("{}\n{}=> ", self.message, formatted_options,);
    }

    pub fn is_default(&self, selection: &str) -> bool {
        selection.to_lowercase() == self.default.0.to_lowercase()
            || selection.to_lowercase() == self.default.1.to_lowercase()
    }
    pub fn prompt(&self) -> PromptResult {
        let default_option_value = self
            .get_option_value(self.default.0)
            .unwrap_or_else(|| self.default.1.to_string());

        //> Print available options
        self.print();

        //> Read the user's input
        let mut input = String::new();
        stdout().flush().expect("Failed to flush stdout.");
        stdin()
            .read_line(&mut input)
            .expect("Failed to read input.");

        input = input.trim().to_string();

        let selection = if input.is_empty() {
            self.default_option_key()
        } else {
            input.clone()
        }
        .to_lowercase();

        // Find the selected option
        let option = self.options.iter().find(|(key, value)| {
            key.to_lowercase() == selection || value.to_lowercase() == selection
        });

        // Return the selected option
        if option.is_none() {
            PromptResult::Error(format!(
                "Invalid option selected. Defaulting to: {}",
                default_option_value
            ))
        } else {
            let selected_option_value = option.unwrap().1.to_string();
            PromptResult::Success(selected_option_value)
        }
    }
}

pub fn test_prompt_with_options() {
    let prompt_options = PromptWithOptions {
        message: "Should we proceed?",
        options: vec![
            ("y", "Yes"),
            ("n", "No"),
            ("a", "Always"),
            ("q", "Quit"),
            ("x", "Exit"),
        ],
        default: ("y", "Yes"),
    };

    match prompt_options.prompt() {
        PromptResult::Success(selection) => {
            println!("Proceeding with option: {}", selection)
        }
        PromptResult::Error(error) => {
            eprintln!("{}", error)
        }
    }
}
