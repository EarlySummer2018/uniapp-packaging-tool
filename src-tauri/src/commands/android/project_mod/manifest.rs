//! AndroidManifest.xml 处理：结构校验修复、组件查找、属性操作、转义工具。

use super::gradle::insert_before_index;
use super::types::{ChildIdentity, EntryIdentity};
use regex::Regex;
use std::path::Path;

/// 判断 AndroidManifest.xml 中 child 标签是否可以作为 parent 的合法子元素。
///
/// 完整的 AndroidManifest 层级规则，用于 validate_manifest_structure 检测非法嵌套。
pub(crate) fn can_be_child_of(child: &str, parent: &str) -> bool {
    match parent {
        "activity" => matches!(child, "intent-filter" | "meta-data" | "layout"),
        "intent-filter" => {
            matches!(child, "action" | "category" | "data" | "mime-type")
        }
        "application" => matches!(
            child,
            "activity"
                | "service"
                | "receiver"
                | "provider"
                | "meta-data"
                | "uses-library"
                | "property"
        ),
        "manifest" => matches!(
            child,
            "application"
                | "permission"
                | "uses-permission"
                | "uses-permission-sdk-23"
                | "uses-feature"
                | "uses-sdk"
                | "query"
                | "supports-screens"
        ),
        _ => true, // 其他父标签不做限制
    }
}

/// 自动修复 XML 内容中未正确闭合或嵌套的标签。
///
/// 在写入 AndroidManifest.xml 前调用，自动补全缺失的 </activity> 等结束标记，
/// 避免 Gradle ManifestMerger2 报出 SAXParseException。
///
/// 修复策略：
/// - 遇到不匹配的结束标签（如 `</application>` 但栈顶是 `<activity>`），
///   在该结束标签前插入缺失的 `</activity>`
/// - 扫描结束后栈中残留的未闭合标签，按逆序在末尾补充结束标记
pub(crate) fn fix_manifest_xml_structure(content: &str) -> Result<String, String> {
    let tag_re = Regex::new(r#"<(/?)([a-zA-Z][\w-]*)[^>]*?(/?)>"#)
        .map_err(|e| format!("XML 校验正则编译失败: {}", e))?;

    /// 栈中记录的未闭合开始标签
    struct OpenTag {
        name: String,
    }

    let mut stack: Vec<OpenTag> = Vec::new();
    let mut fixes: Vec<(usize, String)> = Vec::new(); // (插入位置, 要插入的内容)

    for mat in tag_re.find_iter(content) {
        let caps = tag_re
            .captures(mat.as_str())
            .expect("正则匹配成功，captures 必定存在");
        let is_close = &caps[1] == "/";
        let is_self_closing = caps.get(3).map_or(false, |m| m.as_str() == "/");
        let name = caps[2].to_string();

        if is_close {
            match stack.last() {
                Some(top) if top.name == name => {
                    stack.pop();
                }
                Some(top) => {
                    // 结束标签与栈顶不匹配 → 在当前位置前插入缺失的结束标记
                    let insertion = format!("    </{}>\n", top.name);
                    fixes.push((mat.start(), insertion));
                    stack.pop();
                    // 弹出后继续检查是否还有其他不匹配的
                    let mut retry = true;
                    while retry {
                        match stack.last() {
                            Some(t) if t.name == name => {
                                stack.pop();
                                retry = false;
                            }
                            Some(t) => {
                                let ins = format!("    </{}>\n", t.name);
                                fixes.push((mat.start(), ins));
                                stack.pop();
                            }
                            None => {
                                retry = false;
                            }
                        }
                    }
                }
                None => {
                    // 多余的结束标签，忽略（不删除，保持原样）
                }
            }
        } else if !is_self_closing {
            // 检查 Android Manifest 合法性：某些标签不能作为 <activity> 的子元素
            // 如果栈顶是 <activity> 而当前标签不合法，先闭合 <activity>
            while let Some(top) = stack.last() {
                if !can_be_child_of(&name, &top.name) {
                    let ins = format!("    </{}>\n", top.name);
                    fixes.push((mat.start(), ins));
                    stack.pop();
                } else {
                    break;
                }
            }
            stack.push(OpenTag { name });
        }
    }

    // 栈中剩余的未闭合标签：按逆序在内容末尾补充结束标记
    if !stack.is_empty() {
        let mut tail_fixes = String::new();
        for tag in stack.iter().rev() {
            tail_fixes.push_str(&format!("    </{}>\n", tag.name));
        }
        fixes.push((content.len(), tail_fixes));
    }

    if fixes.is_empty() {
        return Ok(content.to_string());
    }

    // 按位置从后往前应用修复（避免偏移量影响）
    let mut result = content.to_string();
    fixes.sort_by_key(|(pos, _)| std::cmp::Reverse(*pos));
    for (pos, insertion) in &fixes {
        result.insert_str(*pos, insertion);
    }

    Ok(result)
}

/// 在所有工程修改完成后、Gradle 构建前，
/// 对最终 AndroidManifest.xml 进行结构校验和自动修复。
pub fn validate_and_fix_final_manifest(workspace: &Path) -> Result<(), String> {
    use super::modifier::AndroidProjectModifier;
    use super::types::MODULE_NAME;

    let path = workspace
        .join(MODULE_NAME)
        .join("src/main/AndroidManifest.xml");

    let modifier = AndroidProjectModifier::new(workspace.to_path_buf())?;
    let content = modifier.read_file(&path)?;
    let fixed = fix_manifest_xml_structure(&content)?;
    if fixed != content {
        // 有修改才写入，避免不必要的 IO
        modifier.write_file(&path, &fixed)?;
    }
    Ok(())
}

pub(crate) fn set_meta_data_value(
    content: &str,
    name: &str,
    value: &str,
) -> Result<String, String> {
    let Some(mat) = find_xml_start_tag_with_attr(content, "meta-data", "android:name", name) else {
        return Ok(content.to_string());
    };
    let tag = &content[mat.start..mat.end];
    let value_re = Regex::new(r#"android:value\s*=\s*"[^"]*""#).unwrap();
    let updated = if value_re.is_match(tag) {
        value_re
            .replace(
                tag,
                format!(r#"android:value="{}""#, escape_xml_attr(value)),
            )
            .to_string()
    } else {
        tag.trim_end_matches("/>")
            .trim_end_matches('>')
            .trim_end()
            .to_string()
            + &format!(r#" android:value="{}" />"#, escape_xml_attr(value))
    };
    Ok(replace_range(content, mat.start, mat.end, &updated))
}

pub(crate) struct ManifestComponent {
    pub start: usize,
    pub end: usize,
    pub end_close_start: usize,
}

pub(crate) fn find_manifest_component(
    content: &str,
    tag: &str,
    name: &str,
) -> Option<ManifestComponent> {
    let start = find_xml_start_tag_with_attr(content, tag, "android:name", name)?;
    let start_tag = &content[start.start..start.end];
    if start_tag.trim_end().ends_with("/>") {
        return Some(ManifestComponent {
            start: start.start,
            end: start.end,
            end_close_start: start.end - 2,
        });
    }

    // 使用深度匹配定位正确的闭合标签，避免多同名标签时匹配错误位置
    let end_tag = format!("</{}>", tag);
    let open_pattern = format!("<{}", tag);

    let mut depth = 1; // 已进入目标开始标签
    let mut search_pos = start.end;

    while search_pos < content.len() {
        if let Some(rel) = content[search_pos..].find(&end_tag) {
            let close_pos = search_pos + rel;

            // 统计从 search_pos 到 close_pos 之间有多少个同名的开标签
            let segment = &content[search_pos..close_pos];
            let open_count = count_tag_open_occurrences(segment, &open_pattern);

            // 空片段（如 <activity></activity>）不应有嵌套标签
            let actual_open_count = if segment.is_empty() { 0 } else { open_count };

            if depth - actual_open_count == 1 {
                // 这个 </tag> 是正确的配对
                return Some(ManifestComponent {
                    start: start.start,
                    end: close_pos + end_tag.len(),
                    end_close_start: close_pos,
                });
            } else {
                // 这个 </tag> 属于嵌套的同名标签，继续往后找
                depth = depth - 1 + open_count;
                search_pos = close_pos + end_tag.len();
            }
        } else {
            break; // 没找到更多闭合标签
        }
    }
    None
}

/// 计算文本片段中某个标签的开标签出现次数（排除自闭合和闭标签）
pub(crate) fn count_tag_open_occurrences(fragment: &str, tag_pattern: &str) -> usize {
    let re = Regex::new(&format!(
        r"{}\b(?![/])(?:\s|>|[a-z_-])",
        regex::escape(tag_pattern)
    ))
    .unwrap_or_else(|_| Regex::new(r"$^").unwrap()); // 永不匹配的空正则作为 fallback
    re.find_iter(fragment).count()
}

pub(crate) fn entry_identity(entry: &str) -> EntryIdentity {
    if let Some(name) = android_attr_value(entry, "android:name") {
        if entry.contains("<meta-data") {
            return EntryIdentity::MetaData(name);
        }
        for tag in ["activity", "service", "receiver", "provider"] {
            if entry.contains(&format!("<{}", tag)) {
                return EntryIdentity::Component {
                    tag: tag.to_string(),
                    name,
                };
            }
        }
    }
    if let Some(authority) = android_attr_value(entry, "android:authorities") {
        return EntryIdentity::ProviderAuthority(authority);
    }
    if entry.trim_start().starts_with("<!--") {
        return EntryIdentity::Comment(entry.trim().to_string());
    }
    EntryIdentity::Raw(entry.trim().to_string())
}

/// 生成 Manifest 条目的可读描述，用于日志输出，方便定位问题条目
pub(crate) fn format_entry_description(_entry: &str, identity: &EntryIdentity) -> String {
    match identity {
        EntryIdentity::MetaData(name) => format!("meta-data({})", name),
        EntryIdentity::Component { tag, name } => format!("{}({})", tag, name),
        EntryIdentity::ProviderAuthority(auth) => {
            format!("provider(authorities={})", auth)
        }
        EntryIdentity::Comment(text) => {
            if text.len() > 50 {
                format!("comment({}...)", &text[..50])
            } else {
                format!("comment({})", text)
            }
        }
        EntryIdentity::Raw(raw) => {
            if raw.len() > 50 {
                format!("raw({}...)", &raw[..50])
            } else {
                format!("raw({})", raw)
            }
        }
    }
}

pub(crate) fn child_identity(child: &str) -> ChildIdentity {
    if let Some(scheme) = android_attr_value(child, "android:scheme") {
        return ChildIdentity::IntentFilterDataScheme(scheme);
    }
    if let Some(action) = android_attr_value(child, "android:name") {
        return ChildIdentity::IntentFilterAction(action);
    }
    ChildIdentity::Raw(child.trim().to_string())
}

/// 判断一个 Manifest 条目是否为裸的 <data> 自闭合元素。
///
/// <data> 元素在 AndroidManifest 中必须作为 <intent-filter> 的子元素出现，
/// 不能直接放在 <application> 下。如果检测到这种条目，需要特殊路由处理。
pub(crate) fn is_bare_data_element(entry: &str) -> bool {
    let trimmed = entry.trim();
    // 匹配 <data ... /> 形式的自闭合标签（可能是条目的唯一内容）
    let data_re = Regex::new(r#"^\s*<data\b[^>]*/>\s*$"#).unwrap();
    if data_re.is_match(trimmed) {
        return true;
    }
    // 也匹配包含多个标签的片段中，顶层只有 <data ... /> 的情况
    let lines: Vec<&str> = trimmed.lines().collect();
    let non_empty: Vec<&str> = lines
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("<!--"))
        .collect();
    non_empty.len() == 1 && data_re.is_match(non_empty[0])
}

/// 将裸 <data> 元素路由到目标 Activity 的 intent-filter 内部。
///
/// 策略：
/// 1. 从 <data> 的 android:scheme 属性推断关联的模块（如 WX_APPID → wxapi.WXEntryActivity）
/// 2. 在 manifest 中查找该 Activity（支持后缀匹配）
/// 3. 如果 Activity 已有 <intent-filter>，将 <data> 插入其中
/// 4. 如果没有 intent-filter 但 Activity 存在，创建一个新的并插入 <data>
pub(crate) fn route_data_to_activity_intent_filter(
    content: &str,
    data_entry: &str,
) -> Result<String, String> {
    let scheme = android_attr_value(data_entry, "android:scheme")
        .ok_or_else(|| "<data> 条目缺少 android:scheme 属性".to_string())?;

    // 根据 scheme 前缀推测目标 Activity 名称（可能是短名称如 wxapi.WXEntryActivity）
    let target_hint = infer_target_activity_from_scheme(&scheme);

    // 在 manifest 中查找该 Activity（优先精确匹配，再尝试后缀匹配）
    let activity = find_manifest_component(content, "activity", &target_hint)
        .or_else(|| find_manifest_component_suffix(content, "activity", &target_hint))
        .ok_or_else(|| format!("未找到目标 Activity: {}（已尝试后缀匹配）", target_hint))?;

    // 检查 Activity 内是否已有 intent-filter
    let activity_body = &content[activity.start..activity.end];
    let has_intent_filter = activity_body.contains("<intent-filter");

    let indented_data = indent_xml_fragment(data_entry, 12);

    if has_intent_filter {
        // 找到最后一个 </intent-filter> 的位置（相对于 content），在其前面插入 <data>
        let body_start = activity.start;
        let body_end = activity.end_close_start;

        let body = &content[body_start..body_end];
        let mut last_if_close = None;
        let if_close_pattern = "</intent-filter>";
        let mut search_pos = 0;
        while let Some(rel) = body[search_pos..].find(if_close_pattern) {
            last_if_close = Some(body_start + search_pos + rel);
            search_pos += rel + if_close_pattern.len();
        }

        let insert_pos = last_if_close.ok_or("Activity 中找不到 </intent-filter>".to_string())?;
        Ok(insert_before_index(
            content,
            insert_pos,
            &format!("\n{}", indented_data),
        ))
    } else {
        // Activity 没有 intent-filter，在 </activity> 前创建一个新的
        let insert_pos = activity.end_close_start; // </activity> 的位置
        Ok(insert_before_index(
            content,
            insert_pos,
            &format!(
                "\n        <intent-filter>\n{}\n        </intent-filter>",
                indented_data
            ),
        ))
    }
}

/// 通过 android:name 属性的后缀匹配查找 Manifest 组件。
///
/// 当只知道类名的后半部分（如 wxapi.WXEntryActivity）而不知道完整包名时使用。
pub(crate) fn find_manifest_component_suffix(
    content: &str,
    tag: &str,
    name_suffix: &str,
) -> Option<ManifestComponent> {
    if name_suffix.is_empty() {
        return None;
    }
    let re = Regex::new(&format!(
        r#"(?s)<{}\b[^>]*android:name="[^"]*{}"[^>]*>"#,
        regex::escape(tag),
        regex::escape(name_suffix)
    ))
    .ok()?;

    let mat = re.find(content)?;
    let start_tag = &content[mat.start()..mat.end()];
    let start = mat.start();

    if start_tag.trim_end().ends_with("/>") {
        return Some(ManifestComponent {
            start,
            end: mat.end(),
            end_close_start: mat.end() - 2,
        });
    }

    let end_tag = format!("</{}>", tag);
    let open_pattern = format!("<{}", tag);

    let mut depth = 1;
    let mut search_pos = mat.end();

    while search_pos < content.len() {
        if let Some(rel) = content[search_pos..].find(&end_tag) {
            let close_pos = search_pos + rel;
            let segment = &content[search_pos..close_pos];
            let open_count = count_tag_open_occurrences(segment, &open_pattern);

            if depth - open_count == 1 {
                return Some(ManifestComponent {
                    start,
                    end: close_pos + end_tag.len(),
                    end_close_start: close_pos,
                });
            } else {
                depth = depth - 1 + open_count;
                search_pos = close_pos + end_tag.len();
            }
        } else {
            break;
        }
    }
    None
}

/// 从 <data android:scheme="..."> 的 scheme 值推断对应的目标 Activity 全限定名。
///
/// 常见映射：
/// - ${WX_APPID} / wx...  → PACKAGE.wxapi.WXEntryActivity
/// - tencent{QQ_APPID}    → com.tencent.tauth.AuthActivity
/// - 其他                  → 返回 Err 让调用方回退到普通插入
fn infer_target_activity_from_scheme(scheme: &str) -> String {
    // scheme 通常是模板占位符如 ${WX_APPID} 或实际值
    // 通过 module_application_entries 中已有的 activity 条目来反推
    // 这里做基于常见约定的启发式匹配

    // 微信相关 scheme → WXEntryActivity
    if scheme.contains("WX_APPID") || scheme.starts_with("wx") || scheme.contains("wxa") {
        // 返回通配符模式，由调用方在 manifest 中模糊匹配
        // 实际上我们需要找到已存在的 wxapi.WXEntryActivity
        return "wxapi.WXEntryActivity".to_string();
    }

    // QQ 相关
    if scheme.contains("QQ_APPID") || scheme.starts_with("tencent") {
        return "com.tencent.tauth.AuthActivity".to_string();
    }

    // 新浪微博
    if scheme.contains("SINA") || scheme.contains("sina") {
        return "com.sina.weibo.sdk.share.WbShareTransActivity".to_string();
    }

    // 默认：尝试从所有已知 activity 中找包含 intent-filter 的那个
    // 这里返回一个通用标识，让 route_data_to_activity_intent_filter 做模糊搜索
    String::new()
}

pub(crate) fn find_xml_start_tag_with_attr(
    content: &str,
    tag: &str,
    attr: &str,
    value: &str,
) -> Option<ManifestComponent> {
    let re = Regex::new(&format!(r#"(?s)<{}\b[^>]*>"#, regex::escape(tag))).ok()?;
    for mat in re.find_iter(content) {
        let fragment = mat.as_str();
        if android_attr_value(fragment, attr).as_deref() == Some(value) {
            return Some(ManifestComponent {
                start: mat.start(),
                end: mat.end(),
                end_close_start: mat.end(),
            });
        }
    }
    None
}

pub(crate) fn android_attr_value(fragment: &str, attr: &str) -> Option<String> {
    let re = Regex::new(&format!(r#"{}\s*=\s*"([^"]*)""#, regex::escape(attr))).ok()?;
    re.captures(fragment)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().to_string())
}

pub(crate) fn set_string_resource(
    content: &str,
    name: &str,
    value: &str,
) -> Result<String, String> {
    let re = Regex::new(&format!(
        r#"(?s)<string\s+name="{}">.*?</string>"#,
        regex::escape(name)
    ))
    .unwrap();
    let replacement = format!(
        r#"<string name="{}">{}</string>"#,
        name,
        escape_xml_text(value)
    );
    if re.is_match(content) {
        Ok(re.replace(content, replacement).to_string())
    } else {
        let close = content
            .rfind("</resources>")
            .ok_or_else(|| "strings.xml 缺少 </resources>".to_string())?;
        Ok(insert_before_index(
            content,
            close,
            &format!("    {}\n", replacement),
        ))
    }
}

pub(crate) fn replace_range(content: &str, start: usize, end: usize, replacement: &str) -> String {
    let mut result = String::with_capacity(content.len() + replacement.len());
    result.push_str(&content[..start]);
    result.push_str(replacement);
    result.push_str(&content[end..]);
    result
}

pub(crate) fn indent_xml_fragment(fragment: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    fragment
        .trim()
        .lines()
        .map(|line| format!("{}{}", indent, line.trim()))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn escape_xml_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(crate) fn escape_xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
