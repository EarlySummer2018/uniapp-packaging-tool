use super::*;

#[test]
fn manifest_modules_generate_dcloud_properties() {
    let modules = vec![
        DetectedModule {
            name: "OAuth".to_string(),
            category: "login".to_string(),
            platforms: vec!["all".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        },
        DetectedModule {
            name: "Payment".to_string(),
            category: "payment".to_string(),
            platforms: vec!["all".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        },
        DetectedModule {
            name: "Share".to_string(),
            category: "share".to_string(),
            platforms: vec!["all".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        },
    ];
    let config = module_config_from_detected_modules(&modules);
    // 测试用：传入空白名单（不限制），验证所有模块都能正确生成
    let enabled: Vec<String> = vec![];
    let path = temp_file("unipack-dcloud-properties");

    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(content
        .contains(r#"<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl""#));
    assert!(content.contains(
        r#"<module name="OAuth-Weixin" value="io.dcloud.feature.oauth.weixin.WeiXinOAuthService"/>"#
    ));
    assert!(content.contains(
        r#"<feature name="Payment" value="io.dcloud.feature.payment.PaymentFeatureImpl">"#
    ));
    assert!(content
        .contains(r#"<module name="AliPay" value="io.dcloud.feature.payment.alipay.AliPay"/>"#));
    assert!(content.contains(r#"<feature name="Share""#));
    assert_eq!(content.matches("<features>").count(), 1);
    assert_eq!(content.matches("</features>").count(), 1);

    let _ = std::fs::remove_file(path);
}

#[test]
fn dcloud_properties_payment_replaces_defaults_and_includes_all_providers() {
    let path = temp_file("unipack-all-payment-properties");
    std::fs::write(
        &path,
        r#"<properties>
    <features>
        <feature name="Payment"><module name="AliPay"/></feature>
    </features>
</properties>"#,
    )
    .unwrap();
    let config = ModuleConfigTree {
        payment: Some(PaymentModuleConfig {
            enabled: true,
            weixin: Some(HashMap::new()),
            alipay: Some(HashMap::new()),
            paypal: Some(HashMap::new()),
            stripe: Some(HashMap::new()),
            google: Some(HashMap::new()),
        }),
        ..Default::default()
    };

    generate_dcloud_properties(&path, &config, &["Payment".to_string()]).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    for provider in [
        "AliPay",
        "Payment-Weixin",
        "Payment-Paypal",
        "Payment-Stripe",
        "Payment-Google",
    ] {
        assert!(content.contains(&format!(r#"<module name="{}""#, provider)));
    }
    assert_eq!(content.matches(r#"<feature name="Payment""#).count(), 1);

    let _ = std::fs::remove_file(path);
}

#[test]
fn dcloud_properties_camera_only_does_not_add_disabled_features() {
    let path = temp_file("unipack-camera-only-properties");
    let config = module_config_with_camera_share_oauth_payment();
    let enabled = vec!["Camera".to_string()];

    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(content
        .contains(r#"<feature name="Camera" value="io.dcloud.js.camera.CameraFeatureImpl"/>"#));
    assert!(!content.contains(r#"<feature name="Share""#));
    assert!(!content.contains(r#"<feature name="OAuth""#));
    assert!(!content.contains(r#"<feature name="Login""#));
    assert!(!content.contains(r#"<feature name="Payment""#));
    assert!(!content.contains(r#"<feature name="Ad""#));

    let _ = std::fs::remove_file(path);
}

#[test]
fn app_plus_other_modules_are_reported_and_generate_properties() {
    let project_root =
        std::env::temp_dir().join(format!("unipack-other-modules-{}", uuid::Uuid::new_v4()));
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "VideoPlayer": {},
                "Barcode": {},
                "Bluetooth": {},
                "iBeacon": {},
                "Contacts": {},
                "Fingerprint": {},
                "Messaging": {},
                "Record": {},
                "SQLite": {},
                "gcanvas": {},
                "Webview-x5": {}
            }
        }
    });
    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );
    let report =
        android_module_config_report_from_value(&info.detected_modules, Some(&manifest), None);
    let template_keys = report
        .modules
        .iter()
        .map(|module| module.template_key.as_str())
        .collect::<Vec<_>>();

    for key in [
        "video_player",
        "barcode",
        "bluetooth",
        "ibeacon",
        "contacts",
        "fingerprint",
        "messaging",
        "record",
        "sqlite",
        "gcanvas",
        "x5_tbs",
    ] {
        assert!(template_keys.contains(&key), "{key} should be reported");
    }
    assert!(report
        .modules
        .iter()
        .filter(|module| module.template_key != "livepusher")
        .all(|module| module.fields.is_empty()));

    let config = module_config_from_detected_modules(&info.detected_modules);
    let enabled = info
        .detected_modules
        .iter()
        .map(|module| module.name.clone())
        .collect::<Vec<_>>();
    let path = temp_file("unipack-other-modules-properties");

    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    for feature in [
        r#"<feature name="VideoPlayer" value="io.dcloud.media.MediaFeatureImpl"/>"#,
        r#"<feature name="Barcode" value="io.dcloud.feature.barcode2.BarcodeFeatureImpl"/>"#,
        r#"<feature name="Bluetooth" value="io.dcloud.feature.bluetooth.BluetoothFeature"/>"#,
        r#"<feature name="iBeacon" value="io.dcloud.feature.iBeacon.WxBluetoothFeatureImpl"/>"#,
        r#"<feature name="Contacts" value="io.dcloud.feature.contacts.ContactsFeatureImpl"/>"#,
        r#"<feature name="Fingerprint" value="io.dcloud.feature.fingerprint.FingerPrintsImpl"/>"#,
        r#"<feature name="Messaging" value="io.dcloud.adapter.messaging.MessagingPluginImpl"/>"#,
        r#"<feature name="Sqlite" value="io.dcloud.feature.sqlite.DataBaseFeature"/>"#,
        r#"<feature name="X5Webview" value="io.dcloud.feature.X5Webview.X5WebViewService"/>"#,
    ] {
        assert!(content.contains(feature), "{feature} should be generated");
    }

    let _ = std::fs::remove_file(path);
}

#[test]
fn dcloud_properties_push_adds_feature_and_service() {
    let path = temp_file("unipack-push-properties");
    let mut config = ModuleConfigTree::default();
    config.push = Some(PushModuleConfig {
        enabled: true,
        unipush_appid: Some("demo-appid".to_string()),
        unipush_appkey: Some("demo-appkey".to_string()),
        unipush_appsecret: Some("demo-secret".to_string()),
        vendors: Vec::new(),
    });
    let enabled = vec!["Push".to_string()];

    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(
        content.contains(r#"<feature name="Push" value="io.dcloud.feature.aps.APSFeatureImpl">"#)
    );
    assert!(content
        .contains(r#"<module name="unipush" value="io.dcloud.feature.unipush.GTPushService"/>"#));
    assert!(
        content.contains(r#"<service name="push" value="io.dcloud.feature.aps.APSFeatureImpl"/>"#)
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn dcloud_properties_umeng_statistic_replaces_template_defaults() {
    let path = temp_file("unipack-umeng-statistic-properties");
    std::fs::write(
        &path,
        r#"<properties>
	<features>
		<feature name="Statistic" value="io.dcloud.feature.statistics.StatisticsFeatureImpl"/>
	</features>
	<services>
		<service name="Statistic" value="io.dcloud.feature.statistics.StatisticsBootImpl"/>
	</services>
</properties>"#,
    )
    .unwrap();
    let mut config = ModuleConfigTree::default();
    config.statistic = Some(StatisticModuleConfig {
        enabled: true,
        provider: "umeng".to_string(),
        umeng: Some(HashMap::new()),
        mta: None,
        baidu: None,
    });
    let enabled = vec!["Statistic".to_string()];

    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(content.contains(
        r#"<feature name="Statistic" value="io.dcloud.feature.statistics.StatisticsFeatureImpl">"#
    ));
    assert!(content.contains(
        r#"<module name="Statistic-Umeng" value="io.dcloud.feature.statistics.umeng.UmengStatistics"/>"#
    ));
    assert!(content.contains(
        r#"<service name="Statistic-Umeng" value="io.dcloud.feature.statistics.umeng.StatisticsBootImpl"/>"#
    ));
    assert!(!content.contains(r#"<service name="Statistic" "#));
    assert_eq!(content.matches(r#"feature name="Statistic""#).count(), 1);
    assert_eq!(
        content.matches(r#"service name="Statistic-Umeng""#).count(),
        1
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn dcloud_properties_adds_canonical_oauth_and_share_features() {
    let path = temp_file("unipack-oauth-share-properties");
    let mut config = ModuleConfigTree::default();
    config.login = Some(LoginModuleConfig {
        enabled: true,
        providers: vec![LoginProvider {
            name: "weixin".to_string(),
            enabled: true,
            config: HashMap::new(),
        }],
    });
    config.share = Some(ShareModuleConfig {
        enabled: true,
        weixin: Some(HashMap::new()),
        qq: Some(HashMap::new()),
        sina: None,
    });
    let enabled = vec!["OAuth".to_string(), "Share".to_string()];

    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(content
        .contains(r#"<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl""#));
    assert!(content.contains(
        r#"<module name="OAuth-Weixin" value="io.dcloud.feature.oauth.weixin.WeiXinOAuthService"/>"#
    ));
    assert!(!content.contains(r#"<feature name="Login""#));
    assert!(content.contains(r#"<feature name="Share" value="io.dcloud.share.ShareFeatureImpl""#));
    assert!(
        content.contains(r#"<module name="Weixin" value="io.dcloud.share.mm.WeiXinApiManager"/>"#)
    );
    assert!(content.contains(r#"<module name="QQ" value="io.dcloud.share.qq.QQApiManager"/>"#));

    let _ = std::fs::remove_file(path);
}

#[test]
fn dcloud_properties_generation_is_idempotent_and_treats_login_as_oauth() {
    let path = temp_file("unipack-idempotent-properties");
    std::fs::write(
        &path,
        r#"<properties>
	<features>
		<feature name="Login" value="io.dcloud.feature.login.LoginFeatureImpl"/>
	</features>
</properties>"#,
    )
    .unwrap();
    let mut config = ModuleConfigTree::default();
    config.login = Some(LoginModuleConfig {
        enabled: true,
        providers: vec![LoginProvider {
            name: "weixin".to_string(),
            enabled: true,
            config: HashMap::new(),
        }],
    });
    let enabled = vec!["OAuth".to_string()];

    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert_eq!(content.matches(r#"feature name="Login""#).count(), 1);
    assert_eq!(content.matches(r#"feature name="OAuth""#).count(), 0);

    let _ = std::fs::remove_file(path);
}
