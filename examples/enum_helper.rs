use regex::Regex;
use std::env;
use std::fs;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let content = fs::read_to_string(format!("enum_txt/{}", &args[1]))?;
    let lines = content.split("\n").collect::<Vec<&str>>();
    for &line in lines.iter() {
        let parts = line.split("=").collect::<Vec<&str>>();
        println!("{} = {},", sanitize_enum_variant_name(parts[1]), parts[0].trim())
    }
    Ok(())
}

fn sanitize_enum_variant_name(input: &str) -> String {
    let re = Regex::new(r"[a-zA-Z0-9]+").unwrap();
    let mut result = String::new();
    for cap in re.find_iter(input) {
        let word = cap.as_str();
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_ascii_uppercase());
            result.push_str(chars.as_str());
        }
    }

    if result.chars().next().map_or(false, |c| c.is_ascii_digit()) {
        result.insert(0, '_');
    }

    result
}