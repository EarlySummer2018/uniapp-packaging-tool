use crate::commands::android::project_mod::manifest::entry_identity;
use crate::commands::android::project_mod::xml_editor::XmlManifestEditor;

#[test]
fn application_tools_replace_merges_existing_attributes_idempotently() {
    let mut editor = XmlManifestEditor::from_str(
        r#"<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    xmlns:tools="http://schemas.android.com/tools">
    <application
        android:allowBackup="false"
        tools:replace="android:theme">
    </application>
</manifest>
"#,
    );

    editor
        .add_application_tools_replace("android:allowBackup")
        .unwrap();
    editor
        .add_application_tools_replace("android:allowBackup")
        .unwrap();

    assert!(editor
        .as_str()
        .contains(r#"tools:replace="android:theme,android:allowBackup""#));
    assert_eq!(editor.as_str().matches("xmlns:tools=").count(), 1);
    assert_eq!(editor.as_str().matches("android:allowBackup").count(), 2);
    editor.validate_structure().unwrap();
}

#[test]
fn manifest_editor_updates_existing_metadata_with_tools_replace() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <application>
        <meta-data android:name="GETUI_APPID" android:value="" />
    </application>
</manifest>
"#;
    let mut editor = XmlManifestEditor::from_str(xml);
    let entry = r#"<meta-data android:name="GETUI_APPID" android:value="${GY_APP_ID}" tools:replace="android:value" />"#;

    let inserted = editor
        .add_application_entry(entry, &entry_identity(entry))
        .unwrap();

    assert!(!inserted, "已有同名 meta-data 时应更新而不是重复插入");
    let manifest = editor.as_str();
    assert_eq!(manifest.matches(r#"android:name="GETUI_APPID""#).count(), 1);
    assert!(manifest.contains(r#"android:value="${GY_APP_ID}""#));
    assert!(manifest.contains(r#"tools:replace="android:value""#));
    assert!(manifest.contains(r#"xmlns:tools="http://schemas.android.com/tools""#));
    editor.validate_structure().unwrap();
}
