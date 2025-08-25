use std::fmt::Display;

pub fn option_to_code_str<T: Display>(vec: Vec<Option<T>>) -> Vec<String> {
    vec.into_iter().map(|x| match x {
        None => "None".to_string(),
        Some(x) => format!("Some({})", x)
    }).collect()
}


