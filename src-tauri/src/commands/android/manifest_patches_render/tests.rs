use super::*;
use crate::commands::shared::module::types::{
    AndroidModuleConfigField, AndroidModuleConfigModule, AndroidModuleConfigReport,
};

fn valid_push_manifest() -> serde_json::Value {
    serde_json::json!({
        "app-plus": {
            "modules": {
                "Push": {}
            },
            "distribute": {
                "push": {
                    "unipush": {
                        "version": "2"
                    }
                }
            }
        }
    })
}

#[test]
fn payment_manifest_patches_include_alipay_permissions_and_wechat_entries() {
    let report = AndroidModuleConfigReport {
        modules: vec![AndroidModuleConfigModule {
            name: "Payment".to_string(),
            template_key: "payment".to_string(),
            category: "payment".to_string(),
            platforms: vec!["android".to_string()],
            source: "manifest.json".to_string(),
            fields: vec![AndroidModuleConfigField {
                key: "WX_APPID".to_string(),
                value: Some("wx-demo".to_string()),
                ..Default::default()
            }],
        }],
        all_configured: true,
        ..Default::default()
    };

    let (patches, _) =
        render_android_module_manifest_patches_impl(Some(&report), "com.example.demo", "");

    for permission in [
        "android.permission.INTERNET",
        "android.permission.ACCESS_NETWORK_STATE",
        "android.permission.ACCESS_WIFI_STATE",
        "android.permission.READ_PHONE_STATE",
        "android.permission.WRITE_EXTERNAL_STORAGE",
        "android.permission.ACCESS_COARSE_LOCATION",
        "android.permission.MODIFY_AUDIO_SETTINGS",
    ] {
        assert!(patches.permissions.contains(permission));
    }
    assert!(patches
        .application_entries
        .contains("io.dcloud.feature.payment.weixin.WXPayProcessMeadiatorActivity"));
    assert!(patches
        .application_entries
        .contains("com.example.demo.wxapi.WXPayEntryActivity"));
}

#[test]
fn all_selected_payment_providers_include_official_manifest_entries() {
    let report = AndroidModuleConfigReport {
        modules: vec![AndroidModuleConfigModule {
            name: "Payment".to_string(),
            template_key: "payment".to_string(),
            category: "payment".to_string(),
            platforms: vec!["android".to_string()],
            source: "manifest.json".to_string(),
            fields: vec![
                AndroidModuleConfigField {
                    key: "WX_APPID".to_string(),
                    value: Some("wx-demo".to_string()),
                    ..Default::default()
                },
                AndroidModuleConfigField {
                    key: "PAYPAL_RETURN_SCHEME".to_string(),
                    value: Some("paypal-demo".to_string()),
                    ..Default::default()
                },
            ],
        }],
        all_configured: true,
        ..Default::default()
    };
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "payment": {
                        "alipay": {},
                        "weixin": {},
                        "paypal": {},
                        "stripe": {},
                        "google": {}
                    }
                }
            }
        }
    });

    let (patches, _) = render_android_module_manifest_patches_for_manifest_impl(
        Some(&report),
        Some(&manifest),
        "com.example.demo",
        "",
    );

    for expected in [
        "io.dcloud.feature.payment.weixin.WXPayProcessMeadiatorActivity",
        "@style/ProjectDialogTheme",
        "com.example.demo.wxapi.WXPayEntryActivity",
        "com.paypal.openid.RedirectUriReceiverActivity",
        "com.paypal.pyplcheckout.home.view.activities.PYPLInitiateCheckoutActivity",
        r#"android:scheme="${PAYPAL_RETURN_SCHEME}""#,
        r#"android:value="${PAYPAL_RETURN_SCHEME}://paypalpay""#,
        "io.dcloud.feature.payment.stripe.TransparentActivity",
        "com.google.android.gms.wallet.api.enabled",
    ] {
        assert!(patches.application_entries.contains(expected), "{expected}");
    }
}

#[test]
fn payment_manifest_entries_follow_selected_providers() {
    let report = AndroidModuleConfigReport {
        modules: vec![AndroidModuleConfigModule {
            name: "Payment".to_string(),
            template_key: "payment".to_string(),
            category: "payment".to_string(),
            platforms: vec!["android".to_string()],
            source: "manifest.json".to_string(),
            fields: Vec::new(),
        }],
        all_configured: true,
        ..Default::default()
    };
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "payment": {
                        "alipay": {}
                    }
                }
            }
        }
    });

    let (patches, _) = render_android_module_manifest_patches_for_manifest_impl(
        Some(&report),
        Some(&manifest),
        "com.example.demo",
        "",
    );

    assert!(patches.permissions.contains("android.permission.INTERNET"));
    for unexpected in [
        "WXPayEntryActivity",
        "RedirectUriReceiverActivity",
        "TransparentActivity",
        "com.google.android.gms.wallet.api.enabled",
    ] {
        assert!(
            !patches.application_entries.contains(unexpected),
            "{unexpected}"
        );
    }
    assert!(!patches
        .permissions
        .contains("android.permission.MODIFY_AUDIO_SETTINGS"));
}

#[test]
fn system_geolocation_adds_documented_base_permissions_only() {
    let report = AndroidModuleConfigReport {
        modules: vec![AndroidModuleConfigModule {
            name: "Geolocation".to_string(),
            template_key: "geolocation".to_string(),
            category: "geolocation".to_string(),
            platforms: vec!["android".to_string()],
            source: "manifest.json".to_string(),
            fields: Vec::new(),
        }],
        all_configured: true,
        ..Default::default()
    };

    let (patches, _) =
        render_android_module_manifest_patches_impl(Some(&report), "com.example.demo", "");

    for permission in [
        "android.permission.ACCESS_COARSE_LOCATION",
        "android.permission.ACCESS_FINE_LOCATION",
        "android.permission.ACCESS_WIFI_STATE",
        "android.permission.ACCESS_NETWORK_STATE",
        "android.permission.CHANGE_WIFI_STATE",
        "android.permission.READ_PHONE_STATE",
        "android.permission.WRITE_EXTERNAL_STORAGE",
        "android.permission.INTERNET",
        "android.permission.MOUNT_UNMOUNT_FILESYSTEMS",
        "android.permission.READ_LOGS",
        "android.permission.WRITE_SETTINGS",
    ] {
        assert!(patches.permissions.contains(permission));
    }
    assert!(!patches
        .permissions
        .contains("android.permission.ACCESS_BACKGROUND_LOCATION"));
    assert!(!patches
        .permissions
        .contains("android.permission.FOREGROUND_SERVICE"));
    assert!(!patches.application_entries.contains("APSService"));
}

#[test]
fn amap_geolocation_adds_amap_manifest_entries_and_extra_permissions() {
    let report = AndroidModuleConfigReport {
        modules: vec![AndroidModuleConfigModule {
            name: "Geolocation".to_string(),
            template_key: "geolocation".to_string(),
            category: "geolocation".to_string(),
            platforms: vec!["android".to_string()],
            source: "manifest.json".to_string(),
            fields: vec![AndroidModuleConfigField {
                key: "AMAP_KEY".to_string(),
                value: Some("amap-demo".to_string()),
                ..Default::default()
            }],
        }],
        all_configured: true,
        ..Default::default()
    };

    let (patches, _) =
        render_android_module_manifest_patches_impl(Some(&report), "com.example.demo", "");

    assert!(patches
        .permissions
        .contains("android.permission.ACCESS_BACKGROUND_LOCATION"));
    assert!(patches
        .permissions
        .contains("android.permission.FOREGROUND_SERVICE"));
    assert!(patches
        .application_entries
        .contains("com.amap.api.v2.apikey"));
    assert!(patches.application_entries.contains("APSService"));
}

#[test]
fn payment_template_uses_wx_pay_entry_activity_source() {
    let template = crate::commands::module::get_module_template_sync("payment").unwrap();

    assert!(template
        .android_config
        .activities
        .iter()
        .any(|activity| activity.starts_with(".wxapi.WXPayEntryActivity")));
    assert!(!template
        .android_config
        .activities
        .iter()
        .any(|activity| activity.starts_with(".wxapi.WXPayActivity")));
}

#[test]
fn other_modules_manifest_patches_include_required_permissions() {
    let report = AndroidModuleConfigReport {
        modules: [
            ("Barcode", "barcode"),
            ("Bluetooth", "bluetooth"),
            ("iBeacon", "ibeacon"),
            ("Contacts", "contacts"),
            ("Fingerprint", "fingerprint"),
            ("Messaging", "messaging"),
            ("Record", "record"),
            ("LivePusher", "livepusher"),
        ]
        .into_iter()
        .map(|(name, template_key)| AndroidModuleConfigModule {
            name: name.to_string(),
            template_key: template_key.to_string(),
            category: template_key.to_string(),
            platforms: vec!["android".to_string()],
            source: "manifest.json".to_string(),
            fields: Vec::new(),
        })
        .collect(),
        all_configured: true,
        ..Default::default()
    };

    let (patches, groups) =
        render_android_module_manifest_patches_impl(Some(&report), "com.example.demo", "");

    for permission in [
        "android.permission.CAMERA",
        "android.permission.VIBRATE",
        "android.permission.FLASHLIGHT",
        "android.permission.ACCESS_FINE_LOCATION",
        "android.permission.BLUETOOTH_CONNECT",
        "android.permission.READ_CONTACTS",
        "android.permission.USE_FINGERPRINT",
        "android.permission.RECEIVE_SMS",
        "android.permission.RECORD_AUDIO",
        "android.permission.INTERNET",
    ] {
        assert!(patches.permissions.contains(permission));
    }
    assert!(patches
        .permissions
        .contains(r#"<uses-feature android:name="android.hardware.camera.autofocus" />"#));
    assert!(groups.iter().any(|group| group.module_name == "barcode"));
    assert!(groups.iter().any(|group| group.module_name == "bluetooth"));
    assert!(groups.iter().any(|group| group.module_name == "livepusher"));
}

#[test]
fn livepusher_alone_includes_documented_permissions_and_features() {
    let report = AndroidModuleConfigReport {
        modules: vec![AndroidModuleConfigModule {
            name: "LivePusher".to_string(),
            template_key: "livepusher".to_string(),
            category: "livepusher".to_string(),
            platforms: vec!["android".to_string()],
            source: "manifest.json".to_string(),
            fields: Vec::new(),
        }],
        all_configured: true,
        ..Default::default()
    };

    let (patches, groups) =
        render_android_module_manifest_patches_impl(Some(&report), "com.example.demo", "");

    for permission in [
        "android.permission.INTERNET",
        "android.permission.ACCESS_NETWORK_STATE",
        "android.permission.ACCESS_WIFI_STATE",
        "android.permission.WRITE_EXTERNAL_STORAGE",
        "android.permission.READ_EXTERNAL_STORAGE",
        "android.permission.RECORD_AUDIO",
        "android.permission.MODIFY_AUDIO_SETTINGS",
        "android.permission.BLUETOOTH",
        "android.permission.CAMERA",
        "android.permission.READ_PHONE_STATE",
    ] {
        assert!(patches.permissions.contains(permission));
    }
    for feature in [
        r#"<uses-feature android:name="android.hardware.Camera" />"#,
        r#"<uses-feature android:name="android.hardware.camera.autofocus" />"#,
    ] {
        assert!(patches.permissions.contains(feature));
    }

    let group = groups
        .iter()
        .find(|group| group.module_name == "livepusher")
        .expect("livepusher patch group");
    assert_eq!(group.permissions.len(), 12);
}

#[test]
fn push_manifest_patches_include_unipush_and_oppo_intent_filters() {
    let report = AndroidModuleConfigReport {
        modules: vec![AndroidModuleConfigModule {
            name: "Push".to_string(),
            template_key: "push".to_string(),
            category: "push".to_string(),
            platforms: vec!["android".to_string()],
            source: "manifest.json".to_string(),
            fields: vec![
                AndroidModuleConfigField {
                    key: "GETUI_APPID".to_string(),
                    value: Some("push-appid".to_string()),
                    ..Default::default()
                },
                AndroidModuleConfigField {
                    key: "plus.unipush.appkey".to_string(),
                    value: Some("push-appkey".to_string()),
                    ..Default::default()
                },
                AndroidModuleConfigField {
                    key: "plus.unipush.appsecret".to_string(),
                    value: Some("push-secret".to_string()),
                    ..Default::default()
                },
                AndroidModuleConfigField {
                    key: "OPPO_APP_KEY".to_string(),
                    value: Some("oppo-key".to_string()),
                    ..Default::default()
                },
            ],
        }],
        all_configured: true,
        ..Default::default()
    };

    let manifest = valid_push_manifest();
    let (patches, _) = render_android_module_manifest_patches_for_manifest_impl(
        Some(&report),
        Some(&manifest),
        "com.example.demo",
        "",
    );

    assert!(patches
        .pandora_entry_intent_filters
        .contains("android:host=\"io.dcloud.unipush\""));
    assert!(patches
        .pandora_entry_intent_filters
        .contains("android.intent.action.oppopush"));
}

#[test]
fn push_manifest_entries_use_alias_placeholders() {
    let report = AndroidModuleConfigReport {
        modules: vec![AndroidModuleConfigModule {
            name: "Push".to_string(),
            template_key: "push".to_string(),
            category: "push".to_string(),
            platforms: vec!["android".to_string()],
            source: "manifest.json".to_string(),
            fields: vec![AndroidModuleConfigField {
                key: "GETUI_APPID".to_string(),
                value: Some("push-appid".to_string()),
                ..Default::default()
            }],
        }],
        all_configured: true,
        ..Default::default()
    };

    let manifest = valid_push_manifest();
    let (patches, _) = render_android_module_manifest_patches_for_manifest_impl(
        Some(&report),
        Some(&manifest),
        "com.example.demo",
        "",
    );

    assert!(patches.application_entries.contains(
            r#"<meta-data android:name="plus.unipush.appid" android:value="${plus.unipush.appid}" tools:replace="android:value" />"#
        ));
    assert!(!patches.application_entries.contains(
        r#"android:name="plus.unipush.appid" android:value="" tools:replace="android:value" />"#
    ));
}

#[test]
fn push_manifest_patches_are_skipped_without_unipush_v2_gate() {
    let report = AndroidModuleConfigReport {
        modules: vec![AndroidModuleConfigModule {
            name: "Push".to_string(),
            template_key: "push".to_string(),
            category: "push".to_string(),
            platforms: vec!["android".to_string()],
            source: "manifest.json".to_string(),
            fields: vec![AndroidModuleConfigField {
                key: "GETUI_APPID".to_string(),
                value: Some("push-appid".to_string()),
                ..Default::default()
            }],
        }],
        all_configured: true,
        ..Default::default()
    };
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Push": {}
            },
            "distribute": {
                "push": {
                    "unipush": {
                        "version": "1"
                    }
                }
            }
        }
    });

    let (patches, groups) = render_android_module_manifest_patches_for_manifest_impl(
        Some(&report),
        Some(&manifest),
        "com.example.demo",
        "",
    );

    assert!(!patches.application_entries.contains("plus.unipush"));
    assert!(!patches
        .pandora_entry_intent_filters
        .contains("io.dcloud.unipush"));
    assert!(!groups.iter().any(|group| group.module_name == "push"));
}

#[test]
fn univerify_getui_metadata_uses_gy_placeholder_and_replaces_aar_value() {
    let report = AndroidModuleConfigReport {
        modules: vec![AndroidModuleConfigModule {
            name: "OAuth".to_string(),
            template_key: "login".to_string(),
            category: "oauth".to_string(),
            platforms: vec!["android".to_string()],
            source: "manifest.json".to_string(),
            fields: vec![AndroidModuleConfigField {
                key: "GY_APP_ID".to_string(),
                value: Some("univerify-appid".to_string()),
                ..Default::default()
            }],
        }],
        all_configured: true,
        ..Default::default()
    };

    let (patches, groups) =
        render_android_module_manifest_patches_impl(Some(&report), "com.example.demo", "");

    assert!(patches.application_entries.contains(
            r#"<meta-data android:name="GETUI_APPID" android:value="${GY_APP_ID}" tools:replace="android:value" />"#
        ));
    assert!(!patches.application_entries.contains(
        r#"android:name="GETUI_APPID" android:value="" tools:replace="android:value" />"#
    ));
    let login_group = groups
        .iter()
        .find(|group| group.module_name == "login")
        .expect("login manifest patch group should exist");
    assert!(login_group.application_entries.iter().any(|entry| {
        entry.contains(r#"android:name="GETUI_APPID""#)
            && entry.contains(r#"android:value="${GY_APP_ID}""#)
            && entry.contains(r#"tools:replace="android:value""#)
    }));
}

#[test]
fn share_sina_manifest_entries_follow_official_config() {
    let report = AndroidModuleConfigReport {
        modules: vec![AndroidModuleConfigModule {
            name: "Share".to_string(),
            template_key: "share".to_string(),
            category: "share".to_string(),
            platforms: vec!["android".to_string()],
            source: "manifest.json".to_string(),
            fields: ["SINA_APPKEY", "SINA_SECRET", "SINA_REDIRECT_URI"]
                .into_iter()
                .map(|key| AndroidModuleConfigField {
                    key: key.to_string(),
                    value: Some(format!("{}-value", key)),
                    ..Default::default()
                })
                .collect(),
        }],
        all_configured: true,
        ..Default::default()
    };

    let (patches, groups) =
        render_android_module_manifest_patches_impl(Some(&report), "com.example.demo", "");

    for permission in [
        "android.permission.CHANGE_WIFI_STATE",
        "android.permission.ACCESS_WIFI_STATE",
        "android.permission.ACCESS_NETWORK_STATE",
    ] {
        assert!(patches.permissions.contains(permission));
    }
    for expected in [
        r#"<meta-data android:name="SINA_APPKEY" android:value="_${SINA_APPKEY}""#,
        r#"<meta-data android:name="SINA_SECRET" android:value="${SINA_SECRET}""#,
        r#"<meta-data android:name="SINA_REDIRECT_URI" android:value="${SINA_REDIRECT_URI}""#,
        "com.sina.weibo.sdk.web.WeiboSdkWebActivity",
        r#"<activity android:name="com.sina.weibo.sdk.share.WbShareTransActivity" android:exported="true""#,
        "com.sina.weibo.sdk.action.ACTION_SDK_REQ_ACTIVITY",
    ] {
        assert!(patches.application_entries.contains(expected), "{expected}");
    }

    let share_group = groups
        .iter()
        .find(|group| group.module_name == "share")
        .expect("share manifest patch group");
    assert!(share_group
        .application_entries
        .iter()
        .any(|entry| entry.contains("ACTION_SDK_REQ_ACTIVITY")));
}

#[test]
fn oauth_manifest_entries_follow_current_android_and_official_value_formats() {
    let report = AndroidModuleConfigReport {
        modules: vec![AndroidModuleConfigModule {
            name: "OAuth".to_string(),
            template_key: "login".to_string(),
            category: "oauth".to_string(),
            platforms: vec!["android".to_string()],
            source: "manifest.json".to_string(),
            fields: ["QQ_APPID", "SINA_APPKEY", "SINA_REDIRECT_URI", "MIUI_APPID"]
                .into_iter()
                .map(|key| AndroidModuleConfigField {
                    key: key.to_string(),
                    value: Some(format!("{}-value", key)),
                    ..Default::default()
                })
                .collect(),
        }],
        all_configured: true,
        ..Default::default()
    };

    let (patches, _) =
        render_android_module_manifest_patches_impl(Some(&report), "com.example.demo", "");

    assert!(patches.application_entries.contains(
        r#"<activity android:name="com.tencent.tauth.AuthActivity" android:exported="true""#
    ));
    assert!(patches.application_entries.contains(
            r#"<activity android:name="com.tencent.connect.common.AssistActivity" android:theme="@android:style/Theme.Translucent.NoTitleBar" android:screenOrientation="portrait" />"#
        ));
    assert!(!patches
        .application_entries
        .contains(r#"android:screenOrientation="behind""#));
    assert!(patches
        .application_entries
        .contains(r#"<meta-data android:name="SINA_APPKEY" android:value="_${SINA_APPKEY}""#));
    assert!(patches
        .application_entries
        .contains(r#"<meta-data android:name="MIUI_APPID" android:value="_${MIUI_APPID}""#));
    assert!(patches
        .permissions
        .contains("com.xiaomi.permission.AUTH_SERVICE"));
}

#[test]
fn oauth_without_miui_does_not_add_xiaomi_auth_permission() {
    let report = AndroidModuleConfigReport {
        modules: vec![AndroidModuleConfigModule {
            name: "OAuth".to_string(),
            template_key: "login".to_string(),
            category: "oauth".to_string(),
            platforms: vec!["android".to_string()],
            source: "manifest.json".to_string(),
            fields: vec![AndroidModuleConfigField {
                key: "QQ_APPID".to_string(),
                value: Some("qq-value".to_string()),
                ..Default::default()
            }],
        }],
        all_configured: true,
        ..Default::default()
    };

    let (patches, groups) =
        render_android_module_manifest_patches_impl(Some(&report), "com.example.demo", "");

    assert!(!patches
        .permissions
        .contains("com.xiaomi.permission.AUTH_SERVICE"));
    let login_group = groups
        .iter()
        .find(|group| group.module_name == "login")
        .expect("login manifest patch group");
    assert!(!login_group
        .permissions
        .iter()
        .any(|permission| permission == "com.xiaomi.permission.AUTH_SERVICE"));
}
