//! 支付模块 (payment) manifest 补丁
//!
//! 支持支付宝、微信支付、PayPal、Stripe、Google Pay。

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::commands::android::project_mod::ManifestPatchGroup;
use crate::commands::shared::module::types::AndroidModuleConfigModule;

use super::helpers::*;

pub fn render_patches(
    module: &AndroidModuleConfigModule,
    permissions: &mut BTreeSet<String>,
    application_entries: &mut BTreeSet<String>,
    _pandora_filters: &mut BTreeSet<String>,
    placeholders: &HashMap<String, String>,
    package_name: &str,
    patch_groups: &mut BTreeMap<String, ManifestPatchGroup>,
) {
    let mut mod_perms: Vec<String> = Vec::new();
    let mut mod_entries: Vec<String> = Vec::new();

    add_permissions(
        permissions,
        &[
            "android.permission.INTERNET",
            "android.permission.ACCESS_NETWORK_STATE",
            "android.permission.ACCESS_WIFI_STATE",
            "android.permission.READ_PHONE_STATE",
            "android.permission.WRITE_EXTERNAL_STORAGE",
            "android.permission.ACCESS_COARSE_LOCATION",
            "android.permission.MODIFY_AUDIO_SETTINGS",
        ],
    );
    mod_perms.extend([
        "android.permission.INTERNET".to_string(),
        "android.permission.ACCESS_NETWORK_STATE".to_string(),
        "android.permission.ACCESS_WIFI_STATE".to_string(),
        "android.permission.READ_PHONE_STATE".to_string(),
        "android.permission.WRITE_EXTERNAL_STORAGE".to_string(),
        "android.permission.ACCESS_COARSE_LOCATION".to_string(),
        "android.permission.MODIFY_AUDIO_SETTINGS".to_string(),
    ]);

    if has_report_value(module, "WX_APPID") {
        let wx_pay_entries = [
            meta_data("WX_APPID", &placeholder_value(placeholders, "WX_APPID")),
            service_entry(
                r#"<activity android:name="io.dcloud.feature.payment.weixin.WXPayProcessMeadiatorActivity" android:exported="false" android:excludeFromRecents="true" android:theme="@style/ProjectDialogTheme" />"#,
            ),
            service_entry(&format!(
                r#"<activity android:name="{}.wxapi.WXPayEntryActivity" android:exported="true" android:theme="@android:style/Theme.Translucent.NoTitleBar" android:launchMode="singleTop" />"#,
                package_name
            )),
        ];
        add_application_entries(application_entries, &wx_pay_entries);
        mod_entries.extend(wx_pay_entries.iter().cloned());
    }
    if has_report_value(module, "PAYPAL_RETURN_SCHEME") {
        let scheme = placeholder_value(placeholders, "PAYPAL_RETURN_SCHEME");
        let paypal_entries = [
            service_entry(&format!(
                r#"<activity android:name="com.paypal.openid.RedirectUriReceiverActivity" android:excludeFromRecents="true" android:exported="true" android:theme="@style/PYPLAppTheme">
    <intent-filter>
        <action android:name="android.intent.action.VIEW" />
        <category android:name="android.intent.category.DEFAULT" />
        <category android:name="android.intent.category.BROWSABLE" />
        <data android:host="paypalpay" android:scheme="{}" />
    </intent-filter>
</activity>"#,
                scheme
            )),
            service_entry(&format!(
                r#"<activity android:name="com.paypal.pyplcheckout.home.view.activities.PYPLInitiateCheckoutActivity" android:exported="true" android:theme="@style/AppFullScreenTheme">
    <intent-filter android:autoVerify="true">
        <action android:name="android.intent.action.VIEW" />
        <category android:name="android.intent.category.DEFAULT" />
        <category android:name="android.intent.category.BROWSABLE" />
        <data android:host="paypalxo" android:scheme="{}" />
    </intent-filter>
</activity>"#,
                scheme
            )),
            meta_data("returnUrl", &format!("{}://paypalpay", scheme)),
        ];
        add_application_entries(application_entries, &paypal_entries);
        mod_entries.extend(paypal_entries.iter().cloned());
    }
    if has_report_value(module, "androidxVersion") {
        let stripe_entries = [service_entry(
            r#"<activity android:name="io.dcloud.feature.payment.stripe.TransparentActivity" android:excludeFromRecents="true" android:exported="false" android:theme="@style/TranslucentTheme" />"#,
        )];
        add_application_entries(application_entries, &stripe_entries);
        mod_entries.extend(stripe_entries.iter().cloned());
    }

    // 写入 patch group
    let group_name = module.template_key.clone();
    let group = patch_groups
        .entry(group_name.clone())
        .or_insert_with(|| ManifestPatchGroup {
            module_name: group_name,
            permissions: Vec::new(),
            application_entries: Vec::new(),
            intent_filters: Vec::new(),
        });
    group.permissions.extend(mod_perms);
    group.application_entries.extend(mod_entries);
}
