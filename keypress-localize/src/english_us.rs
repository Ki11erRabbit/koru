
pub fn localize_english(input: &str) -> String {
    match input {
        "`" => "~".to_string(),
        "1" => "!".to_string(),
        "2" => "@".to_string(),
        "3" => "#".to_string(),
        "4" => "$".to_string(),
        "5" => "%".to_string(),
        "6" => "^".to_string(),
        "7" => "&".to_string(),
        "8" => "*".to_string(),
        "9" => "(".to_string(),
        "0" => ")".to_string(),
        "-" => "_".to_string(),
        "=" => "+".to_string(),
        "[" => "{".to_string(),
        "]" => "}".to_string(),
        "\\" => "|".to_string(),
        ";" => ":".to_string(),
        "'" => "\"".to_string(),
        "," => "<".to_string(),
        "." => ">".to_string(),
        "/" => "?".to_string(),
        x => x.to_string(),
    }
}