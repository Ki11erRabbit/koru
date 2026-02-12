use std::sync::LazyLock;
use hashbrown::HashMap;
use scheme_rs::exceptions::Exception;
use scheme_rs::num::SimpleNumber;
use scheme_rs::registry::bridge;
use scheme_rs::value::Value;
use tokio::sync::Mutex;
use crate::styled_text::{ColorDefinition, ColorType, ColorValue};

static COLOR_DEFINITION: LazyLock<Mutex<HashMap<ColorType, ColorValue>>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(ColorType::Base, ColorValue::from_hex(0xffffff));
    map.insert(ColorType::SecondaryBase, ColorValue::from_hex(0xe8e8e8));
    map.insert(ColorType::TertiaryBase, ColorValue::from_hex(0xcfcfcf));
    map.insert(ColorType::Surface0, ColorValue::from_hex(0xb3b3b3));
    map.insert(ColorType::Surface1, ColorValue::from_hex(0x989898));
    map.insert(ColorType::Surface2, ColorValue::from_hex(0x7d7d7d));
    map.insert(ColorType::Overlay0, ColorValue::from_hex(0x656565));
    map.insert(ColorType::Overlay1, ColorValue::from_hex(0x484848));
    map.insert(ColorType::Overlay2, ColorValue::from_hex(0x383838));
    map.insert(ColorType::Text, ColorValue::from_hex(0x000000));
    map.insert(ColorType::Subtext0, ColorValue::from_hex(0x333333));
    map.insert(ColorType::Subtext1, ColorValue::from_hex(0x4a4a4a));
    map.insert(ColorType::Subtext1, ColorValue::from_hex(0x4a4a4a));
    map.insert(ColorType::Accent, ColorValue::from_hex(0x74b9fc));
    map.insert(ColorType::Link, ColorValue::from_hex(0x2939f0));
    map.insert(ColorType::Success, ColorValue::from_hex(0x06d61f));
    map.insert(ColorType::Warning, ColorValue::from_hex(0xd69106));
    map.insert(ColorType::Error, ColorValue::from_hex(0xd61006));
    map.insert(ColorType::Tags, ColorValue::from_hex(0x7b00d4));
    map.insert(ColorType::Selection, ColorValue::from_hex(0xf4fa41));
    map.insert(ColorType::Cursor, ColorValue::from_hex(0x808080));
    map.insert(ColorType::SecondaryCursor, ColorValue::from_hex(0x919191));
    map.insert(ColorType::Type, ColorValue::from_hex(0xe8dc35));
    map.insert(ColorType::Interface, ColorValue::from_hex(0xe8dc35));
    map.insert(ColorType::Function, ColorValue::from_hex(0x1e8de3));
    map.insert(ColorType::Method, ColorValue::from_hex(0x1e8de3));
    map.insert(ColorType::Macro, ColorValue::from_hex(0x1e8de3));
    map.insert(ColorType::Keyword, ColorValue::from_hex(0x752da6));
    map.insert(ColorType::Comment, ColorValue::from_hex(0x828282));
    map.insert(ColorType::String, ColorValue::from_hex(0x22e33f));
    map.insert(ColorType::Literal, ColorValue::from_hex(0xdb850d));
    map.insert(ColorType::Operator, ColorValue::from_hex(0x10e88e));
    map.insert(ColorType::Pink, ColorValue::from_hex(0xffd9d9));
    map.insert(ColorType::Red, ColorValue::from_hex(0xf71616));
    map.insert(ColorType::Lime, ColorValue::from_hex(0x4bdb12));
    map.insert(ColorType::Green, ColorValue::from_hex(0x04de04));
    map.insert(ColorType::LightYellow, ColorValue::from_hex(0xfdff75));
    map.insert(ColorType::Yellow, ColorValue::from_hex(0xeff216));
    map.insert(ColorType::Orange, ColorValue::from_hex(0xffa200));
    map.insert(ColorType::Brown, ColorValue::from_hex(0x7a4f04));
    map.insert(ColorType::LightBlue, ColorValue::from_hex(0xc4c9ff));
    map.insert(ColorType::Blue, ColorValue::from_hex(0x2b3ae0));
    map.insert(ColorType::LightMagenta, ColorValue::from_hex(0xfa9eff));
    map.insert(ColorType::Magenta, ColorValue::from_hex(0xf540ff));
    map.insert(ColorType::LightPurple, ColorValue::from_hex(0xd582ff));
    map.insert(ColorType::Purple, ColorValue::from_hex(0xb325fa));
    map.insert(ColorType::LightCyan, ColorValue::from_hex(0x8abdff));
    map.insert(ColorType::Cyan, ColorValue::from_hex(0x2160b0));
    map.insert(ColorType::White, ColorValue::from_hex(0xffffff));
    map.insert(ColorType::LightGray, ColorValue::from_hex(0xbdbdbd));
    map.insert(ColorType::Gray, ColorValue::from_hex(0x8a8a8a));
    map.insert(ColorType::Black, ColorValue::from_hex(0x000000));

    Mutex::new(map)
});

pub async fn all_color_definitions() -> Vec<ColorDefinition> {
    let guard = COLOR_DEFINITION.lock().await;
    let mut output = Vec::with_capacity(guard.len());
    for (key, value) in guard.iter() {
        output.push(ColorDefinition::new(*key, *value));
    }
    output
}

#[bridge(name = "color-definition-hex-set!", lib = "(koru-theme)")]
pub async fn set_color_definition_hex(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((key, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()));
    };
    let Some((value, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(1, args.len()));
    };
    let key: String = key.clone().try_into()?;
    let value: SimpleNumber = value.clone().try_into()?;
    let key = match ColorType::try_from(key.as_str()) {
        Ok(key) => key,
        Err(msg) => {
            return Err(Exception::error(msg));
        }
    };
    let hex: u32 = match value.try_into() {
        Ok(value) => value,
        Err(exn) => {
            return Err(exn);
        }
    };

    let mut guard = COLOR_DEFINITION.lock().await;
    guard.insert(key, ColorValue::from_hex(hex));
    Ok(Vec::new())
}

#[bridge(name = "color-definition-ansi-set!", lib = "(koru-theme)")]
pub async fn set_color_definition_ansi(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((key, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()));
    };
    let Some((value, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(1, args.len()));
    };
    let key: String = key.clone().try_into()?;
    let value: SimpleNumber = value.clone().try_into()?;
    let key = match ColorType::try_from(key.as_str()) {
        Ok(key) => key,
        Err(msg) => {
            return Err(Exception::error(msg));
        }
    };
    let ansi: u8 = match value.try_into() {
        Ok(value) => value,
        Err(exn) => {
            return Err(exn);
        }
    };

    let mut guard = COLOR_DEFINITION.lock().await;
    guard.insert(key, ColorValue::Ansi(ansi));
    Ok(Vec::new())
}