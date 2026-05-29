#![allow(dead_code)]
use anyhow::Result;
use plist::Value;

pub fn read_plist_file(path: &std::path::Path) -> Result<Value> {
    let content = std::fs::read(path)?;
    Value::from_reader(std::io::Cursor::new(content))
        .map_err(|e| anyhow::anyhow!("Failed to parse plist: {}", e))
}

pub fn write_plist_file(path: &std::path::Path, value: &Value) -> Result<()> {
    let mut output = std::fs::File::create(path)?;
    value
        .to_writer_xml(&mut output)
        .map_err(|e| anyhow::anyhow!("Failed to write plist: {}", e))?;
    Ok(())
}

pub fn get_plist_value<'a>(plist: &'a Value, key: &str) -> Option<&'a Value> {
    match plist {
        Value::Dictionary(dict) => dict.get(key),
        _ => None,
    }
}

pub fn set_plist_value(plist: &mut Value, key: &str, value: Value) -> Result<()> {
    match plist {
        Value::Dictionary(dict) => {
            dict.insert(key.to_string(), value);
            Ok(())
        }
        _ => Err(anyhow::anyhow!("Expected a dictionary plist")),
    }
}

pub fn remove_plist_key(plist: &mut Value, key: &str) -> Result<bool> {
    match plist {
        Value::Dictionary(dict) => Ok(dict.remove(key).is_some()),
        _ => Err(anyhow::anyhow!("Expected a dictionary plist")),
    }
}

pub fn create_info_plist(
    bundle_identifier: &str,
    bundle_version: &str,
    bundle_short_version: &str,
    display_name: &str,
    executable_name: &str,
    minimum_os_version: &str,
) -> Value {
    let mut dict = plist::Dictionary::new();
    dict.insert(
        "CFBundleIdentifier".to_string(),
        Value::String(bundle_identifier.to_string()),
    );
    dict.insert(
        "CFBundleVersion".to_string(),
        Value::String(bundle_version.to_string()),
    );
    dict.insert(
        "CFBundleShortVersionString".to_string(),
        Value::String(bundle_short_version.to_string()),
    );
    dict.insert(
        "CFBundleDisplayName".to_string(),
        Value::String(display_name.to_string()),
    );
    dict.insert(
        "CFBundleExecutable".to_string(),
        Value::String(executable_name.to_string()),
    );
    dict.insert(
        "MinimumOSVersion".to_string(),
        Value::String(minimum_os_version.to_string()),
    );
    dict.insert(
        "CFBundlePackageType".to_string(),
        Value::String("APPL".to_string()),
    );
    dict.insert(
        "CFBundleSupportedPlatforms".to_string(),
        Value::Array(vec![Value::String("iPhoneOS".to_string())]),
    );
    dict.insert(
        "UIRequiredDeviceCapabilities".to_string(),
        Value::Array(vec![Value::String("armv7".to_string())]),
    );
    dict.insert(
        "UISupportedInterfaceOrientations".to_string(),
        Value::Array(vec![
            Value::String("UIInterfaceOrientationPortrait".to_string()),
            Value::String("UIInterfaceOrientationLandscapeLeft".to_string()),
            Value::String("UIInterfaceOrientationLandscapeRight".to_string()),
        ]),
    );
    dict.insert(
        "UILaunchStoryboardName".to_string(),
        Value::String("LaunchScreen".to_string()),
    );
    Value::Dictionary(dict)
}
