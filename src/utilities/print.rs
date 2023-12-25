pub fn printres<T: std::fmt::Display, E: std::fmt::Display>(result: Result<T, E>) {
    match result {
        Ok(value) => println!("{}", value),
        Err(error) => eprintln!("{}", error),
    }
}

#[macro_export]
macro_rules! printres {
    ($result:expr) => {
        printres($result)
    };
}

pub fn printres_opt<T: std::fmt::Display, E: std::fmt::Display>(result: Result<Option<T>, E>) {
    match result {
        Ok(Some(value)) => println!("{}", value),
        Ok(None) => println!("None"),
        Err(error) => eprintln!("{}", error),
    }
}

#[macro_export]
macro_rules! printres_opt {
    ($result:expr) => {
        printres_opt($result)
    };
}
