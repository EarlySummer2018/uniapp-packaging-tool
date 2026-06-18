use super::super::support::{ios_payment_alipay_manifest, prepare_ios_payment_alipay_project};
use crate::commands::ios::modules::oauth::apply_ios_oauth_module;
use crate::commands::ios::modules::payment::{apply_ios_payment_module, IosPaymentProvider};
use crate::commands::ios::modules::share::apply_ios_share_module;

#[test]
fn ios_payment_alipay_prefers_framework_when_present() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-payment-alipay-framework-{}",
        uuid::Uuid::new_v4()
    ));
    let (project_root, project_file, libs_dir) = prepare_ios_payment_alipay_project(&root);
    std::fs::create_dir_all(libs_dir.join("AlipaySDK.framework")).unwrap();
    std::fs::create_dir_all(libs_dir.join("AlipaySDK.xcframework")).unwrap();
    let info = ios_payment_alipay_manifest(&root);

    let integration = apply_ios_payment_module(&project_root, &project_file, Some(&info))
        .unwrap()
        .unwrap();

    assert_eq!(integration.providers, vec![IosPaymentProvider::Alipay]);
    assert_eq!(integration.resource_count, 1);
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("../SDK/Libs/AlipaySDK.framework"));
    assert!(!pbxproj.contains("../SDK/Libs/AlipaySDK.xcframework"));
    assert!(project_root.join("AlipaySDK.bundle").is_dir());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_payment_alipay_uses_xcframework_when_framework_missing() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-payment-alipay-xcframework-{}",
        uuid::Uuid::new_v4()
    ));
    let (project_root, project_file, libs_dir) = prepare_ios_payment_alipay_project(&root);
    std::fs::create_dir_all(libs_dir.join("AlipaySDK.xcframework")).unwrap();
    let info = ios_payment_alipay_manifest(&root);

    let integration = apply_ios_payment_module(&project_root, &project_file, Some(&info))
        .unwrap()
        .unwrap();

    assert_eq!(integration.providers, vec![IosPaymentProvider::Alipay]);
    assert_eq!(integration.resource_count, 1);
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("../SDK/Libs/AlipaySDK.xcframework"));
    assert!(pbxproj.contains("lastKnownFileType = wrapper.xcframework"));
    assert!(!pbxproj.contains("../SDK/Libs/AlipaySDK.framework"));
    assert!(project_root.join("AlipaySDK.bundle").is_dir());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_payment_paypal_and_stripe_raise_deployment_target_to_13() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-payment-ios13-{}",
        uuid::Uuid::new_v4()
    ));
    let (project_root, project_file, libs_dir) = prepare_ios_payment_alipay_project(&root);
    for file in ["libpaypalpay.a", "libstripepay.a"] {
        std::fs::write(libs_dir.join(file), "lib").unwrap();
    }
    for xcframework in [
        "PayPalCheckout.xcframework",
        "StripeApplePay.xcframework",
        "StripeCore.xcframework",
        "StripeUICore.xcframework",
        "Stripe3DS2.xcframework",
        "StripePayments.xcframework",
        "StripePaymentsUI.xcframework",
        "StripePaymentSheet.xcframework",
    ] {
        std::fs::create_dir_all(libs_dir.join(xcframework)).unwrap();
    }
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Payment": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "payment": {
                        "paypal": {
                            "__platform__": ["ios"],
                            "returnUrl": "demo.paypal"
                        },
                        "stripe": {
                            "__platform__": ["ios"],
                            "returnUrl": "demo.stripe"
                        }
                    }
                }
            }
        }
    });
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );

    let integration = apply_ios_payment_module(&project_root, &project_file, Some(&info))
        .unwrap()
        .unwrap();

    assert_eq!(
        integration.providers,
        vec![IosPaymentProvider::Paypal, IosPaymentProvider::Stripe]
    );
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("IPHONEOS_DEPLOYMENT_TARGET = 13.0;"));
    assert!(pbxproj.contains("PayPalCheckout.xcframework in Frameworks"));
    assert!(pbxproj.contains("StripePaymentSheet.xcframework in Frameworks"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_weixin_payment_replaces_plain_wechat_sdk_for_share_and_oauth() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-weixin-pay-sdk-dedupe-{}",
        uuid::Uuid::new_v4()
    ));
    let (project_root, project_file, libs_dir) = prepare_ios_payment_alipay_project(&root);
    for file in [
        "liblibOauth.a",
        "libWXOauth.a",
        "liblibShare.a",
        "libweixinShare.a",
        "liblibPayment.a",
        "libwxpay.a",
        "libWeChatSDK_pay.a",
    ] {
        std::fs::write(libs_dir.join(file), "lib").unwrap();
    }
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "OAuth": {},
                "Share": {},
                "Payment": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "oauth": {
                        "weixin": {
                            "__platform__": ["ios"],
                            "appid": "wx-oauth"
                        }
                    },
                    "share": {
                        "weixin": {
                            "__platform__": ["ios"],
                            "appid": "wx-share"
                        }
                    },
                    "payment": {
                        "weixin": {
                            "__platform__": ["ios"],
                            "appid": "wx-pay"
                        }
                    }
                }
            }
        }
    });
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );

    apply_ios_oauth_module(&project_root, &project_file, Some(&info))
        .unwrap()
        .expect("Weixin OAuth should be applied");
    apply_ios_share_module(&project_root, &project_file, Some(&info))
        .unwrap()
        .expect("Weixin Share should be applied");
    apply_ios_payment_module(&project_root, &project_file, Some(&info))
        .unwrap()
        .expect("Weixin Payment should be applied");

    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("libWXOauth.a in Frameworks"));
    assert!(pbxproj.contains("libweixinShare.a in Frameworks"));
    assert!(pbxproj.contains("libwxpay.a in Frameworks"));
    assert!(pbxproj.contains("libWeChatSDK_pay.a in Frameworks"));
    assert!(!pbxproj.contains("libWeChatSDK.a in Frameworks"));
    let _ = std::fs::remove_dir_all(root);
}
