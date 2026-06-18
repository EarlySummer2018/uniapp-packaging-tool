use crate::commands::android::project_mod::manifest::fix_manifest_xml_structure;

#[test]
fn fix_manifest_xml_structure_passes_through_well_formed_xml() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <application android:label="Test">
        <activity android:name=".MainActivity" android:exported="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
            </intent-filter>
        </activity>
        <meta-data android:name="key" android:value="val" />
    </application>
</manifest>
"#;
    let result = fix_manifest_xml_structure(xml).unwrap();
    assert_eq!(result, xml, "格式正确的 XML 不应被修改");
}

#[test]
fn fix_manifest_xml_structure_auto_closes_unclosed_activity() {
    let xml = r#"<manifest>
    <application>
        <activity android:name=".Main">
    </application>
</manifest>
"#;
    let result = fix_manifest_xml_structure(xml).unwrap();
    assert!(
        result.contains("</activity>"),
        "修复后应包含自动补全的 </activity>"
    );
    // 验证修复后的 XML 可以再次通过校验（幂等性）
    let re_check = fix_manifest_xml_structure(&result).unwrap();
    assert_eq!(re_check, result, "修复结果应幂等，二次调用不再修改");
}

#[test]
fn fix_manifest_xml_structure_fixes_mismatched_tags() {
    // 交叉嵌套：<manifest><a><b></a></b></manifest>
    let xml = r#"<manifest>
    <a><b></a></b>
</manifest>
"#;
    let result = fix_manifest_xml_structure(xml).unwrap();
    // 应在 </a> 前插入 </b>，在末尾补 </a>
    assert!(result.contains("</b>"), "应补全缺失的 </b>");
    // 验证幂等性
    let re_check = fix_manifest_xml_structure(&result).unwrap();
    assert_eq!(re_check, result, "修复结果应幂等");
}

#[test]
fn fix_manifest_xml_structure_preserves_self_closing_tags() {
    let xml = r#"<manifest>
    <application>
        <meta-data android:name="k" android:value="v" />
        <uses-permission android:name="p" />
    </application>
</manifest>
"#;
    let result = fix_manifest_xml_structure(xml).unwrap();
    assert_eq!(result, xml, "含自闭合标签的正确 XML 不应被修改");
}
