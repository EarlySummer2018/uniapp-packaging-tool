use std::fs;
use std::path::Path;

use crate::commands::shared::module::templates::android_module_template_key;
use crate::commands::shared::module::types::ModuleConfigTree;

/// 需要在 dcloud_properties.xml 中注册的 feature 定义
struct DCloudPropertyFeature {
    name: String,
    xml_fragment: String,
}

/// 需要在 dcloud_properties.xml 中注册的 service 定义
struct DCloudPropertyService {
    name: String,
    xml_fragment: String,
}

/// 检查 dcloud_properties.xml 中是否已存在指定 name 的 feature
fn feature_exists(content: &str, feature_name: &str) -> bool {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_features = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"features" => in_features = true,
                b"feature" if in_features => {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"name" {
                            if let Ok(value) = attr.unescape_value() {
                                if feature_names_match(value.trim(), feature_name) {
                                    return true;
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            // 处理自闭合标签 <feature name="..." value="..." />
            Ok(Event::Empty(e)) if in_features => {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"name" {
                        if let Ok(value) = attr.unescape_value() {
                            if feature_names_match(value.trim(), feature_name) {
                                return true;
                            }
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                if e.local_name().as_ref() == b"features" {
                    in_features = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    false
}

/// 检查 dcloud_properties.xml 中是否已存在指定 name 的 service
fn service_exists(content: &str, service_name: &str) -> bool {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_services = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"services" => in_services = true,
                b"service" if in_services => {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"name" {
                            if let Ok(value) = attr.unescape_value() {
                                if value.trim() == service_name {
                                    return true;
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(e)) if in_services => {
                if e.local_name().as_ref() == b"service" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"name" {
                            if let Ok(value) = attr.unescape_value() {
                                if value.trim() == service_name {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                if e.local_name().as_ref() == b"services" {
                    in_services = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    false
}

fn canonical_feature_name(name: &str) -> &str {
    match name {
        "Login" | "OAuth" => "OAuth",
        _ => name,
    }
}

fn feature_names_match(existing: &str, expected: &str) -> bool {
    canonical_feature_name(existing) == canonical_feature_name(expected)
}

/// 安全地追加 feature 到 <features> 标签内的正确位置
/// 在 </features> 结束标签之前插入新的 feature
fn append_feature_to_xml(content: &str, feature_xml: &str) -> Result<String, String> {
    let end_tag = "</features>";
    if let Some(pos) = content.rfind(end_tag) {
        let mut result = content.to_string();
        result.insert_str(pos, &format!("\t\t{}\n", feature_xml));
        Ok(result)
    } else {
        let self_closing = "<features/>";
        if let Some(pos) = content.find(self_closing) {
            let mut result = content.to_string();
            result.replace_range(
                pos..(pos + self_closing.len()),
                &format!("<features>\n\t\t{}\n\t</features>", feature_xml),
            );
            Ok(result)
        } else {
            Err("dcloud_properties.xml 格式错误：未找到 <features> 标签".to_string())
        }
    }
}

/// 安全地追加 service 到 <services> 标签内；不存在 services 节点时自动创建
fn append_service_to_xml(content: &str, service_xml: &str) -> Result<String, String> {
    let end_tag = "</services>";
    if let Some(pos) = content.rfind(end_tag) {
        let mut result = content.to_string();
        result.insert_str(pos, &format!("\t\t{}\n", service_xml));
        Ok(result)
    } else {
        let self_closing = "<services/>";
        if let Some(pos) = content.find(self_closing) {
            let mut result = content.to_string();
            result.replace_range(
                pos..(pos + self_closing.len()),
                &format!("<services>\n\t\t{}\n\t</services>", service_xml),
            );
            Ok(result)
        } else if let Some(pos) = content.rfind("</properties>") {
            let mut result = content.to_string();
            result.insert_str(
                pos,
                &format!("\t<services>\n\t\t{}\n\t</services>\n", service_xml),
            );
            Ok(result)
        } else {
            Err("dcloud_properties.xml 格式错误：未找到 <properties> 标签".to_string())
        }
    }
}

fn module_is_enabled(enabled_modules: &[String], template_key: &str) -> bool {
    enabled_modules.is_empty()
        || enabled_modules
            .iter()
            .any(|name| android_module_template_key(name) == Some(template_key))
}

fn push_module_once(
    modules: &mut Vec<(&'static str, &'static str)>,
    name: &'static str,
    value: &'static str,
) {
    if !modules.iter().any(|(existing, _)| *existing == name) {
        modules.push((name, value));
    }
}

fn feature_xml(
    name: &str,
    value: Option<&str>,
    modules: &[(&'static str, &'static str)],
) -> String {
    let value_attr = value
        .map(|value| format!(" value=\"{}\"", value))
        .unwrap_or_default();
    if modules.is_empty() {
        return format!("<feature name=\"{}\"{}/>", name, value_attr);
    }

    let mut xml = format!("<feature name=\"{}\"{}>", name, value_attr);
    for (module_name, module_value) in modules {
        xml.push_str(&format!(
            "\n\t\t<module name=\"{}\" value=\"{}\"/>",
            module_name, module_value
        ));
    }
    xml.push_str("\n\t</feature>");
    xml
}

/// 收集需要追加的 feature 列表
/// enabled_modules: 白名单，只有在此列表中的模块才会被追加（为空时表示不限制）
fn collect_features_to_add(
    config: &ModuleConfigTree,
    enabled_modules: &[String],
) -> Vec<DCloudPropertyFeature> {
    let mut features = Vec::new();

    if module_is_enabled(enabled_modules, "push") {
        if let Some(ref push) = config.push {
            if push.enabled {
                let mut feature =
                    "<feature name=\"Push\" value=\"io.dcloud.feature.aps.APSFeatureImpl\">"
                        .to_string();
                if push.unipush_appid.is_some() || push.unipush_appkey.is_some() {
                    feature.push_str(
                        "\n\t\t<module name=\"unipush\" value=\"io.dcloud.feature.unipush.GTPushService\"/>",
                    );
                    feature.push_str("\n\t");
                }
                feature.push_str("</feature>");
                features.push(DCloudPropertyFeature {
                    name: "Push".to_string(),
                    xml_fragment: feature,
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "share") {
        if let Some(ref share) = config.share {
            if share.enabled {
                let mut modules = Vec::new();
                if share.sina.is_some() {
                    push_module_once(
                        &mut modules,
                        "Sina",
                        "io.dcloud.share.sina.SinaWeiboApiManager",
                    );
                }
                if share.weixin.is_some() {
                    push_module_once(
                        &mut modules,
                        "Weixin",
                        "io.dcloud.share.mm.WeiXinApiManager",
                    );
                }
                if share.qq.is_some() {
                    push_module_once(&mut modules, "QQ", "io.dcloud.share.qq.QQApiManager");
                }
                features.push(DCloudPropertyFeature {
                    name: "Share".to_string(),
                    xml_fragment: feature_xml(
                        "Share",
                        Some("io.dcloud.share.ShareFeatureImpl"),
                        &modules,
                    ),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "map") {
        if let Some(ref map) = config.map {
            if map.enabled {
                let mut feature = "<feature name=\"Maps\">".to_string();
                match map.engine.as_str() {
                    "amap" => feature.push_str("<module name=\"Amap\"/></feature>"),
                    "tencent" => feature.push_str("<module name=\"TencentMap\"/></feature>"),
                    _ => feature.push_str("</feature>"),
                }
                features.push(DCloudPropertyFeature {
                    name: "Maps".to_string(),
                    xml_fragment: feature,
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "login") {
        if let Some(ref login) = config.login {
            if login.enabled {
                let mut modules = Vec::new();
                for provider in &login.providers {
                    if provider.enabled {
                        match provider.name.to_ascii_lowercase().as_str() {
                            "weixin" | "wechat" => push_module_once(
                                &mut modules,
                                "OAuth-Weixin",
                                "io.dcloud.feature.oauth.weixin.WeiXinOAuthService",
                            ),
                            "qq" => push_module_once(
                                &mut modules,
                                "OAuth-QQ",
                                "io.dcloud.feature.oauth.qq.QQOAuthService",
                            ),
                            "sina" | "sinaweibo" | "weibo" => push_module_once(
                                &mut modules,
                                "OAuth-Sina",
                                "io.dcloud.feature.oauth.sina.SinaOAuthService",
                            ),
                            "univerify" | "igetui" | "getui" => push_module_once(
                                &mut modules,
                                "OAuth-IGETui",
                                "io.dcloud.feature.igetui.GeTuiOAuthService",
                            ),
                            "miui" | "xiaomi" => push_module_once(
                                &mut modules,
                                "OAuth-MiUi",
                                "io.dcloud.feature.oauth.miui.MiUiOAuthService",
                            ),
                            "google" => push_module_once(
                                &mut modules,
                                "OAuth-Google",
                                "io.dcloud.feature.google.GoogleOAuthService",
                            ),
                            "facebook" => push_module_once(
                                &mut modules,
                                "OAuth-Facebook",
                                "io.dcloud.feature.facebook.FacebookOAuthService",
                            ),
                            _ => {}
                        }
                    }
                }
                features.push(DCloudPropertyFeature {
                    name: "OAuth".to_string(),
                    xml_fragment: feature_xml(
                        "OAuth",
                        Some("io.dcloud.feature.oauth.OAuthFeatureImpl"),
                        &modules,
                    ),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "payment") {
        if let Some(ref payment) = config.payment {
            if payment.enabled {
                let mut modules = Vec::new();
                if payment.alipay.is_some() {
                    push_module_once(
                        &mut modules,
                        "AliPay",
                        "io.dcloud.feature.payment.alipay.AliPay",
                    );
                }
                if payment.weixin.is_some() {
                    push_module_once(
                        &mut modules,
                        "Payment-Weixin",
                        "io.dcloud.feature.payment.weixin.WeiXinPay",
                    );
                }
                features.push(DCloudPropertyFeature {
                    name: "Payment".to_string(),
                    xml_fragment: feature_xml(
                        "Payment",
                        Some("io.dcloud.feature.payment.PaymentFeatureImpl"),
                        &modules,
                    ),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "speech") {
        if let Some(ref speech) = config.speech {
            if speech.enabled {
                let mut modules = Vec::new();
                match speech.engine.to_ascii_lowercase().as_str() {
                    "xunfei" | "xfyun" | "ifly" | "iflytek" => push_module_once(
                        &mut modules,
                        "iFly",
                        "io.dcloud.feature.speech.IflySpeechEngine",
                    ),
                    "baidu" => push_module_once(
                        &mut modules,
                        "baidu",
                        "io.dcloud.feature.speech.BaiduSpeechEngine",
                    ),
                    _ => {}
                }
                if speech.xfyun.is_some() {
                    push_module_once(
                        &mut modules,
                        "iFly",
                        "io.dcloud.feature.speech.IflySpeechEngine",
                    );
                }
                if speech.baidu.is_some() {
                    push_module_once(
                        &mut modules,
                        "baidu",
                        "io.dcloud.feature.speech.BaiduSpeechEngine",
                    );
                }
                if modules.is_empty() {
                    push_module_once(
                        &mut modules,
                        "iFly",
                        "io.dcloud.feature.speech.IflySpeechEngine",
                    );
                    push_module_once(
                        &mut modules,
                        "baidu",
                        "io.dcloud.feature.speech.BaiduSpeechEngine",
                    );
                }
                features.push(DCloudPropertyFeature {
                    name: "Speech".to_string(),
                    xml_fragment: feature_xml(
                        "Speech",
                        Some("io.dcloud.feature.speech.SpeechFeatureImpl"),
                        &modules,
                    ),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "statistic") {
        if let Some(ref stat) = config.statistic {
            if stat.enabled {
                let provider_module = match stat.provider.as_str() {
                    "umeng" => "Umeng",
                    "mta" => "MTA",
                    "baidu" => "Baidu",
                    _ => "DCloud",
                };
                features.push(DCloudPropertyFeature {
                    name: "Statistic".to_string(),
                    xml_fragment: format!(
                        "<feature name=\"Statistic\"><module name=\"{}\"/></feature>",
                        provider_module
                    ),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "face_recognition") {
        if let Some(ref fr) = config.face_recognition {
            if fr.enabled {
                let provider_module = match fr.provider.as_str() {
                    "dcloud" => "DCloud",
                    "baidu" => "Baidu",
                    "aliyun" => "Aliyun",
                    _ => "DCloud",
                };
                features.push(DCloudPropertyFeature {
                    name: "FaceRecognition".to_string(),
                    xml_fragment: format!(
                        "<feature name=\"FaceRecognition\"><module name=\"{}\"/></feature>",
                        provider_module
                    ),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "uni_ad") {
        if let Some(ref ad) = config.uni_ad {
            if ad.enabled {
                let mut feature = "<feature name=\"UniAD\">".to_string();
                if ad.csj.is_some() {
                    feature.push_str("\n\t\t<module name=\"CSJ\"/>");
                }
                if ad.gdt.is_some() {
                    feature.push_str("\n\t\t<module name=\"GDT\"/>");
                }
                if ad.gromore.is_some() {
                    feature.push_str("\n\t\t<module name=\"Gromore\"/>");
                }
                if ad.admob.is_some() {
                    feature.push_str("\n\t\t<module name=\"AdMob\"/>");
                }
                feature.push_str("\n\t</feature>");
                features.push(DCloudPropertyFeature {
                    name: "UniAD".to_string(),
                    xml_fragment: feature,
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "x5_tbs") {
        if let Some(ref x5) = config.x5_tbs {
            if x5.enabled {
                features.push(DCloudPropertyFeature {
                    name: "X5Webview".to_string(),
                    xml_fragment:
                        "<feature name=\"X5Webview\" value=\"io.dcloud.feature.X5Webview.X5WebViewService\"/>"
                            .to_string(),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "livepusher") {
        if let Some(ref lp) = config.livepusher {
            if lp.enabled {
                features.push(DCloudPropertyFeature {
                    name: "LivePusher".to_string(),
                    xml_fragment:
                        "<feature name=\"LivePusher\" value=\"io.dcloud.media.live.LiveMediaFeatureImpl\"/>"
                            .to_string(),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "camera") {
        if let Some(ref camera) = config.camera {
            if camera.enabled {
                features.push(DCloudPropertyFeature {
                    name: "Camera".to_string(),
                    xml_fragment:
                        "<feature name=\"Camera\" value=\"io.dcloud.js.camera.CameraFeatureImpl\"/>"
                            .to_string(),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "video_player") {
        if let Some(ref video_player) = config.video_player {
            if video_player.enabled {
                features.push(DCloudPropertyFeature {
                    name: "VideoPlayer".to_string(),
                    xml_fragment:
                        "<feature name=\"VideoPlayer\" value=\"io.dcloud.media.MediaFeatureImpl\"/>"
                            .to_string(),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "barcode") {
        if let Some(ref barcode) = config.barcode {
            if barcode.enabled {
                features.push(DCloudPropertyFeature {
                    name: "Barcode".to_string(),
                    xml_fragment:
                        "<feature name=\"Barcode\" value=\"io.dcloud.feature.barcode2.BarcodeFeatureImpl\"/>"
                            .to_string(),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "bluetooth") {
        if let Some(ref bluetooth) = config.bluetooth {
            if bluetooth.enabled {
                features.push(DCloudPropertyFeature {
                    name: "Bluetooth".to_string(),
                    xml_fragment:
                        "<feature name=\"Bluetooth\" value=\"io.dcloud.feature.bluetooth.BluetoothFeature\"/>"
                            .to_string(),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "ibeacon") {
        if let Some(ref ibeacon) = config.ibeacon {
            if ibeacon.enabled {
                features.push(DCloudPropertyFeature {
                    name: "iBeacon".to_string(),
                    xml_fragment:
                        "<feature name=\"iBeacon\" value=\"io.dcloud.feature.iBeacon.WxBluetoothFeatureImpl\"/>"
                            .to_string(),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "contacts") {
        if let Some(ref contacts) = config.contacts {
            if contacts.enabled {
                features.push(DCloudPropertyFeature {
                    name: "Contacts".to_string(),
                    xml_fragment:
                        "<feature name=\"Contacts\" value=\"io.dcloud.feature.contacts.ContactsFeatureImpl\"/>"
                            .to_string(),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "fingerprint") {
        if let Some(ref fingerprint) = config.fingerprint {
            if fingerprint.enabled {
                features.push(DCloudPropertyFeature {
                    name: "Fingerprint".to_string(),
                    xml_fragment:
                        "<feature name=\"Fingerprint\" value=\"io.dcloud.feature.fingerprint.FingerPrintsImpl\"/>"
                            .to_string(),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "messaging") {
        if let Some(ref messaging) = config.messaging {
            if messaging.enabled {
                features.push(DCloudPropertyFeature {
                    name: "Messaging".to_string(),
                    xml_fragment:
                        "<feature name=\"Messaging\" value=\"io.dcloud.adapter.messaging.MessagingPluginImpl\"/>"
                            .to_string(),
                });
            }
        }
    }

    if module_is_enabled(enabled_modules, "sqlite") {
        if let Some(ref sqlite) = config.sqlite {
            if sqlite.enabled {
                features.push(DCloudPropertyFeature {
                    name: "Sqlite".to_string(),
                    xml_fragment:
                        "<feature name=\"Sqlite\" value=\"io.dcloud.feature.sqlite.DataBaseFeature\"/>"
                            .to_string(),
                });
            }
        }
    }

    features
}

/// 收集需要追加的 service 列表
fn collect_services_to_add(
    config: &ModuleConfigTree,
    enabled_modules: &[String],
) -> Vec<DCloudPropertyService> {
    let mut services = Vec::new();

    if module_is_enabled(enabled_modules, "push") {
        if let Some(ref push) = config.push {
            if push.enabled {
                services.push(DCloudPropertyService {
                    name: "push".to_string(),
                    xml_fragment:
                        "<service name=\"push\" value=\"io.dcloud.feature.aps.APSFeatureImpl\"/>"
                            .to_string(),
                });
            }
        }
    }

    services
}

pub fn generate_dcloud_properties(
    path: &Path,
    config: &ModuleConfigTree,
    enabled_modules: &[String],
) -> Result<(), String> {
    // Step 1: 读取现有文件（SDK 原始内容或之前生成的）
    let existing_content = if path.exists() {
        fs::read_to_string(path).map_err(|e| format!("读取 dcloud_properties.xml 失败: {}", e))?
    } else {
        "<properties>\n\t<features>\n\t</features>\n</properties>".to_string()
    };

    // Step 2: 根据用户配置和白名单生成需要追加的 features / services 列表
    let features_to_add = collect_features_to_add(config, enabled_modules);
    let services_to_add = collect_services_to_add(config, enabled_modules);

    // Step 3: 逐个校验并追加（幂等性保证）
    let mut result = existing_content;
    for feature in &features_to_add {
        if !feature_exists(&result, &feature.name) {
            result = append_feature_to_xml(&result, &feature.xml_fragment)
                .map_err(|e| format!("追加 feature {} 失败: {}", feature.name, e))?;
        }
    }
    for service in &services_to_add {
        if !service_exists(&result, &service.name) {
            result = append_service_to_xml(&result, &service.xml_fragment)
                .map_err(|e| format!("追加 service {} 失败: {}", service.name, e))?;
        }
    }

    // Step 4: 写回文件
    fs::write(path, result).map_err(|e| format!("写入 dcloud_properties.xml 失败: {}", e))
}

fn save_push_vendor_config(
    _project_dir: &Path,
    _push: &crate::commands::shared::module::types::PushModuleConfig,
) -> Result<(), String> {
    Ok(())
}

fn extract_project_id(project_dir: &Path) -> Result<String, String> {
    let manifest_path = project_dir.join("manifest.json");
    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Cannot read manifest.json: {}", e))?;
    let json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Cannot parse manifest.json: {}", e))?;
    json.get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "No 'id' field in manifest.json".to_string())
}

#[tauri::command]
pub async fn save_module_config(
    project_path: String,
    config: ModuleConfigTree,
) -> Result<(), String> {
    use std::path::PathBuf;

    let project_dir = PathBuf::from(&project_path);

    let data_dir = project_dir.join("assets").join("data");
    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create assets/data directory: {}", e))?;

    let props_path = data_dir.join("dcloud_properties.xml");
    // 前端保存时不传白名单（空列表 = 不限制），保留向后兼容
    generate_dcloud_properties(&props_path, &config, &[])?;

    if let Some(ref push) = config.push {
        save_push_vendor_config(&project_dir, push)?;
    }

    let project_id = extract_project_id(&project_dir)?;
    let config_path = crate::commands::shared::module::types::get_module_config_path(&project_id);
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    let json_content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize module config: {}", e))?;
    fs::write(&config_path, json_content)
        .map_err(|e| format!("Failed to write module config: {}", e))?;

    Ok(())
}
