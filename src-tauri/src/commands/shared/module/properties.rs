use std::fs;
use std::path::Path;

use crate::commands::shared::module::types::ModuleConfigTree;

pub fn generate_dcloud_properties(path: &Path, config: &ModuleConfigTree) -> Result<(), String> {
    let mut features = Vec::new();
    let mut services = Vec::new();

    if let Some(ref push) = config.push {
        if push.enabled {
            let mut feature =
                "    <feature name=\"Push\" value=\"io.dcloud.feature.aps.APSFeatureImpl\">\n"
                    .to_string();
            if push.unipush_appid.is_some() || push.unipush_appkey.is_some() {
                feature.push_str(
                    "      <module name=\"unipush\" value=\"io.dcloud.feature.unipush.GTPushService\"/>\n",
                );
            }
            feature.push_str("    </feature>\n");
            features.push(feature);
            services.push(
                "    <service name=\"push\" value=\"io.dcloud.feature.aps.APSFeatureImpl\"/>\n"
                    .to_string(),
            );
        }
    }

    if let Some(ref share) = config.share {
        if share.enabled {
            features.push(
                "    <feature name=\"Share\" value=\"io.dcloud.feature.share.ShareFeatureImpl\"/>\n"
                    .to_string(),
            );
        }
    }

    if let Some(ref map) = config.map {
        if map.enabled {
            let mut feature = "    <feature name=\"Maps\">".to_string();
            match map.engine.as_str() {
                "amap" => feature.push_str("<module name=\"Amap\"/></feature>\n"),
                "tencent" => feature.push_str("<module name=\"TencentMap\"/></feature>\n"),
                _ => feature.push_str("</feature>\n"),
            }
            features.push(feature);
        }
    }

    if let Some(ref login) = config.login {
        if login.enabled {
            let mut feature =
                "    <feature name=\"Login\" value=\"io.dcloud.feature.login.LoginFeatureImpl\">\n"
                    .to_string();
            for provider in &login.providers {
                if provider.enabled {
                    match provider.name.as_str() {
                        "weixin" => feature.push_str("      <module name=\"WeixinLogin\"/>\n"),
                        "qq" => feature.push_str("      <module name=\"QQLogin\"/>\n"),
                        "apple" => feature.push_str("      <module name=\"AppleLogin\"/>\n"),
                        "univerify" => feature.push_str("      <module name=\"Univerify\"/>\n"),
                        _ => {}
                    }
                }
            }
            feature.push_str("    </feature>\n");
            features.push(feature);
        }
    }

    if let Some(ref payment) = config.payment {
        if payment.enabled {
            let mut feature = "    <feature name=\"Payment\">\n".to_string();
            if payment.weixin.is_some() {
                feature.push_str("      <module name=\"WeixinPay\"/>\n");
            }
            if payment.alipay.is_some() {
                feature.push_str("      <module name=\"Alipay\"/>\n");
            }
            feature.push_str("    </feature>\n");
            features.push(feature);
        }
    }

    if let Some(ref speech) = config.speech {
        if speech.enabled {
            let engine_module = match speech.engine.as_str() {
                "xunfei" => "Xfyun",
                "baidu" => "Baidu",
                "ali" => "Ali",
                _ => "System",
            };
            features.push(format!(
                "    <feature name=\"Speech\"><module name=\"{}\"/></feature>\n",
                engine_module
            ));
        }
    }

    if let Some(ref stat) = config.statistic {
        if stat.enabled {
            let provider_module = match stat.provider.as_str() {
                "umeng" => "Umeng",
                "mta" => "MTA",
                "baidu" => "Baidu",
                _ => "DCloud",
            };
            features.push(format!(
                "    <feature name=\"Statistic\"><module name=\"{}\"/></feature>\n",
                provider_module
            ));
        }
    }

    if let Some(ref fr) = config.face_recognition {
        if fr.enabled {
            let provider_module = match fr.provider.as_str() {
                "dcloud" => "DCloud",
                "baidu" => "Baidu",
                "aliyun" => "Aliyun",
                _ => "DCloud",
            };
            features.push(format!(
                "    <feature name=\"FaceRecognition\"><module name=\"{}\"/></feature>\n",
                provider_module
            ));
        }
    }

    if let Some(ref ad) = config.uni_ad {
        if ad.enabled {
            let mut feature = "    <feature name=\"UniAD\">\n".to_string();
            if ad.csj.is_some() {
                feature.push_str("      <module name=\"CSJ\"/>\n");
            }
            if ad.gdt.is_some() {
                feature.push_str("      <module name=\"GDT\"/>\n");
            }
            if ad.gromore.is_some() {
                feature.push_str("      <module name=\"Gromore\"/>\n");
            }
            if ad.admob.is_some() {
                feature.push_str("      <module name=\"AdMob\"/>\n");
            }
            feature.push_str("    </feature>\n");
            features.push(feature);
        }
    }

    if let Some(ref x5) = config.x5_tbs {
        if x5.enabled {
            features.push("    <feature name=\"X5Webview\" value=\"io.dcloud.feature.X5Webview.X5WebViewService\"/>\n".to_string());
        }
    }

    if let Some(ref lp) = config.livepusher {
        if lp.enabled {
            features.push("    <feature name=\"LivePusher\"/>\n".to_string());
        }
    }

    let mut xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<dcloud-properties>
"#
    .to_string();
    if !features.is_empty() {
        xml.push_str("  <features>\n");
        for feature in features {
            xml.push_str(&feature);
        }
        xml.push_str("  </features>\n");
    }
    if !services.is_empty() {
        xml.push_str("  <services>\n");
        for service in services {
            xml.push_str(&service);
        }
        xml.push_str("  </services>\n");
    }

    xml.push_str("</dcloud-properties>\n");

    fs::write(path, xml).map_err(|e| format!("Failed to write dcloud_properties.xml: {}", e))
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
    generate_dcloud_properties(&props_path, &config)?;

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
