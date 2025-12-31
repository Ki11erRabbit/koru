mod english_us;
mod english_uk;

#[derive(Clone, Copy)]
pub enum KeyboardRegion {
    EnglishUS,
    EnglishUK,
}


/// This function takes a keyboard region and performs a translation on the key as if the shift key was pressed with that code.
/// 
/// This function first tries to uppercase the string. 
/// If this works, then we return the new string.
/// If it doesn't do anything, we pass it through localization functions to see if it is a symbol to translate.
pub fn canonicalize_keypress(
    language: KeyboardRegion,
    keypress: &str,
) -> String {
    let char = keypress.to_uppercase();

    if char != keypress {
        return char;
    }

    match language {
        KeyboardRegion::EnglishUS => english_us::localize_english(keypress),
        KeyboardRegion::EnglishUK => english_uk::localize_english(keypress),
    }
}