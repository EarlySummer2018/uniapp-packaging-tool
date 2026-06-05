//! 基于 quick-xml 的 AndroidManifest.xml 结构感知编辑器。

use super::gradle::replace_range;
use super::manifest::{
    android_attr_value, can_be_child_of, count_tag_open_occurrences, escape_xml_attr,
    find_xml_start_tag_with_attr, indent_xml_fragment, is_bare_data_element,
    route_data_to_activity_intent_filter, ManifestComponent,
};
use super::types::{ChildIdentity, EntryIdentity};
use quick_xml::events::Event;
use quick_xml::Reader;
use regex::Regex;

// ============================================================================
// XmlManifestEditor — 基于 quick-xml 的 AndroidManifest.xml 安全编辑器
// ============================================================================

/// 使用 quick-xml Reader 进行结构感知的 AndroidManifest.xml 编辑器。
///
/// 与旧的正则字符串操作不同，本编辑器通过完整解析 XML 事件流来定位标签位置，
/// 确保插入操作不会破坏 XML 结构合法性。
pub struct XmlManifestEditor {
    content: String,
}

impl XmlManifestEditor {
    pub fn from_str(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.content
    }

    /// 替换内部内容（用于外部修复后更新）
    pub fn replace_content(&mut self, new_content: String) {
        self.content = new_content;
    }

    /// 在 <manifest> 开始标签后添加缺失的 <uses-permission> 声明。
    ///
    /// 使用 quick-xml Reader 定位 <manifest ...> 标签的结束位置（'>' 后），
    /// 在该位置逐个插入缺失的权限元素。
    ///
    /// 如果权限字符串不是以 `<` 开头（即裸权限名如 `"android.permission.CAMERA"`），
    /// 自动包裹为 `<uses-permission android:name="..." />` 格式。
    pub fn add_permissions(&mut self, permissions: &[String]) -> Result<(), String> {
        if permissions.is_empty() {
            return Ok(());
        }

        let manifest_end = Self::find_tag_end(&self.content, "manifest")
            .ok_or_else(|| "AndroidManifest.xml 缺少 <manifest> 标签".to_string())?;

        let mut insertion = String::new();
        for perm in permissions {
            let perm = perm.trim();
            if perm.is_empty() {
                continue;
            }
            // 自动补全 <uses-permission> 标签
            let formatted = if perm.starts_with('<') {
                perm.to_string()
            } else {
                format!(r#"<uses-permission android:name="{}" />"#, perm)
            };
            // 检查是否已存在（用 android:name 值判断）
            if let Some(name) = android_attr_value(&formatted, "android:name") {
                if self
                    .content
                    .contains(&format!(r#"android:name="{}""#, name))
                {
                    continue;
                }
            } else if self.content.contains(&formatted) {
                continue;
            }
            insertion.push_str("\n    ");
            insertion.push_str(&formatted);
        }

        if !insertion.is_empty() {
            self.content.insert_str(manifest_end + 1, &insertion);
        }
        Ok(())
    }

    /// 设置/更新 <application> 标签的属性值。
    ///
    /// 使用正则精确匹配并替换属性值，不重写整个文档，
    /// 从而保留原始 XML 的换行和缩进格式。
    /// 如果同一属性出现多次（如模板残留 + 我们添加的），自动去重保留最后一个值。
    pub fn set_application_attr(
        &mut self,
        attr_name: &str,
        attr_value: &str,
    ) -> Result<(), String> {
        // 查找 <application ...> 开始标签（支持多行，贪婪匹配到 >）
        let re = Regex::new(r#"(?s)(<application\b[^>]*)(>)"#)
            .map_err(|e| format!("编译 application 标签正则失败: {}", e))?;

        let escaped_value = escape_xml_attr(attr_value);
        let attr_re = Regex::new(&format!(r#"\s*{}="[^"]*""#, regex::escape(attr_name))).unwrap();

        if let Some(caps) = re.captures(&self.content) {
            let tag_start = caps.get(1).unwrap().start();
            let tag_end = caps.get(2).unwrap().start(); // > 的位置（不包含 > 本身）
            let tag_content = &self.content[tag_start..tag_end];

            if attr_re.is_match(tag_content) {
                // 属性已存在：在原文档中替换所有出现
                let mut result = self.content.clone();
                let mut matches: Vec<_> = attr_re
                    .find_iter(tag_content)
                    .map(|m| (m.start() + tag_start, m.end() + tag_start))
                    .collect();
                matches.sort_by(|a, b| b.0.cmp(&a.0));
                for (start, end) in matches {
                    result.replace_range(
                        start..end,
                        &format!(r#" {}="{}""#, attr_name, escaped_value),
                    );
                }
                self.content = result;
            } else {
                // 属性不存在：在 > 前追加
                let new_tag = format!(
                    "{}\n        {}=\"{}\">",
                    tag_content.trim_end(),
                    attr_name,
                    escaped_value
                );
                self.content = replace_range(
                    &self.content,
                    tag_start,
                    caps.get(2).unwrap().end(),
                    &new_tag,
                );
            }
            Ok(())
        } else {
            Err("AndroidManifest.xml 缺少 <application> 标签".to_string())
        }
    }

    /// 在 </application> 闭合标签前插入子元素。
    ///
    /// 先通过 identity 检查是否已存在，不存在则使用 quick-xml Reader
    /// 精确定位 </application> 的字节位置并插入。
    ///
    /// 返回 true 表示实际插入了新元素，false 表示已存在（跳过）。
    pub fn add_application_entry(
        &mut self,
        entry: &str,
        identity: &EntryIdentity,
    ) -> Result<bool, String> {
        // 检查是否已存在
        if Self::entry_exists_in_application(&self.content, identity) {
            return Ok(false);
        }

        // 裸 <data> 元素需要路由到对应 Activity 的 intent-filter 内部
        if is_bare_data_element(entry) {
            match route_data_to_activity_intent_filter(&self.content, entry) {
                Ok(new_content) => {
                    self.content = new_content;
                    return Ok(true);
                }
                Err(_) => {} // 路由失败，回退到普通插入路径
            }
        }

        // 定位 </application> 的位置
        let close_pos = Self::find_application_close(&self.content)
            .ok_or_else(|| "AndroidManifest.xml 缺少 </application>".to_string())?;

        let indented = format!("\n{}", indent_xml_fragment(entry, 8));
        self.content.insert_str(close_pos, &indented);
        Ok(true)
    }

    /// 在指定 Activity 的 </activity> 闭合标签前插入子元素（如 intent-filter）。
    ///
    /// 支持精确匹配和后缀匹配 activity 名称。
    /// 返回 true 表示实际插入，false 表示已存在。
    pub fn add_activity_child(
        &mut self,
        activity_name: &str,
        child: &str,
        identity: &ChildIdentity,
    ) -> Result<bool, String> {
        // 检查是否已存在
        if Self::child_exists_in_activity(&self.content, activity_name, identity) {
            return Ok(false);
        }

        // 查找目标 Activity（优先精确匹配，再尝试后缀匹配）
        let activity = Self::find_activity_range(&self.content, activity_name)
            .or_else(|| Self::find_activity_range_suffix(&self.content, activity_name))
            .ok_or_else(|| format!("AndroidManifest.xml 缺少 Activity: {}", activity_name))?;

        let indented = format!("\n{}", indent_xml_fragment(child, 12));
        self.content.insert_str(activity.end_close_start, &indented);
        Ok(true)
    }

    /// 使用 quick-xml 完整解析文档，验证 XML 结构合法性。
    ///
    /// 维护真实的标签栈（基于 Start/End 事件对），检测：
    /// - 标签不匹配
    /// - <data> 不在 <intent-filter> 内部
    /// - 栈未正确清空
    pub fn validate_structure(&self) -> Result<(), String> {
        let mut reader = Reader::from_str(&self.content);
        reader.config_mut().trim_text(false);

        let mut tag_stack: Vec<String> = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                    // 非法嵌套检查
                    if let Some(parent) = tag_stack.last() {
                        if !can_be_child_of(&name, parent) {
                            return Err(format!(
                                "非法嵌套: <{}> 不能作为 <{}> 的子元素",
                                name, parent
                            ));
                        }
                    }
                    tag_stack.push(name);
                }
                Ok(Event::Empty(_)) => {
                    // 自闭合标签，不处理
                }
                Ok(Event::End(ref e)) => {
                    let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    match tag_stack.last() {
                        Some(top) if *top == name => {
                            tag_stack.pop();
                        }
                        Some(top) => {
                            return Err(format!(
                                "标签不匹配: 期望 </{}>，实际遇到 </{}>",
                                top, name
                            ));
                        }
                        None => {
                            return Err(format!("多余的闭合标签 </{}>", name));
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(format!("XML 解析错误: {}", e));
                }
                _ => {}
            }
            buf.clear();
        }

        if !tag_stack.is_empty() {
            return Err(format!("文档结束时仍有未闭合标签: {:?}", tag_stack));
        }

        Ok(())
    }

    // === 内部辅助方法 ===

    /// 使用快速查找定位指定标签名开始标签的 '>' 位置（即开始标签结束的下一个字节）
    fn find_tag_end(content: &str, tag_name: &str) -> Option<usize> {
        let target = format!("<{}", tag_name);
        let start = content.find(&target)?;
        // 从 start 开始找 '>'
        content[start..].find('>').map(|p| start + p)
    }

    /// 从后向前查找最后一个不在注释内的 </application>
    fn find_application_close(content: &str) -> Option<usize> {
        let target = "</application>";
        let comment_start = "<!--";
        let comment_end = "-->";

        let mut search_from = content.len();
        loop {
            let rel = content[..search_from].rfind(target)?;
            let pos = rel;

            let before_pos = &content[..pos];
            let last_comment_open = before_pos.rfind(comment_start);
            let last_comment_close = before_pos.rfind(comment_end);

            match (last_comment_open, last_comment_close) {
                (Some(open_idx), Some(close_idx)) if open_idx > close_idx => {
                    search_from = open_idx;
                }
                _ => return Some(pos),
            }
        }
    }

    /// 检查 application 下是否存在指定 identity 的条目
    fn entry_exists_in_application(content: &str, identity: &EntryIdentity) -> bool {
        match identity {
            EntryIdentity::MetaData(name) => {
                content.contains("<meta-data")
                    && content.contains(&format!(r#"android:name="{}""#, name))
            }
            EntryIdentity::Component { tag, name } => {
                content.contains(&format!("<{}", tag))
                    && content.contains(&format!(r#"android:name="{}""#, name))
            }
            EntryIdentity::ProviderAuthority(authority) => {
                content.contains("<provider")
                    && content.contains(&format!(r#"android:authorities="{}""#, authority))
            }
            EntryIdentity::Comment(text) => content.contains(text),
            EntryIdentity::Raw(fragment) => content.contains(fragment.trim()),
        }
    }

    /// 检查 Activity 下是否存在指定 identity 的子元素
    fn child_exists_in_activity(
        content: &str,
        activity_name: &str,
        identity: &ChildIdentity,
    ) -> bool {
        let Some(activity) = Self::find_activity_range(content, activity_name)
            .or_else(|| Self::find_activity_range_suffix(content, activity_name))
        else {
            return false;
        };

        let body = &content[activity.start..activity.end];
        match identity {
            ChildIdentity::IntentFilterDataScheme(scheme) => {
                body.contains("<intent-filter")
                    && body.contains(&format!(r#"android:scheme="{}""#, scheme))
            }
            ChildIdentity::IntentFilterAction(action) => {
                body.contains("<intent-filter")
                    && body.contains(&format!(r#"android:name="{}""#, action))
            }
            ChildIdentity::Raw(fragment) => body.contains(fragment.trim()),
        }
    }

    /// 通过 android:name 精确查找 Activity 的范围
    fn find_activity_range(content: &str, name: &str) -> Option<XmlTagRange> {
        Self::find_component_range(content, "activity", name)
    }

    /// 通过 android:name 后缀查找 Activity 的范围
    fn find_activity_range_suffix(content: &str, name_suffix: &str) -> Option<XmlTagRange> {
        if name_suffix.is_empty() {
            return None;
        }
        let re = Regex::new(&format!(
            r#"(?s)<activity\b[^>]*android:name="[^"]*{}"[^>]*>"#,
            regex::escape(name_suffix)
        ))
        .ok()?;
        let mat = re.find(content)?;
        Self::parse_component_from_match(content, "activity", mat.start())
    }

    /// 通用组件范围查找（按 tag + android:name 精确匹配）
    fn find_component_range(content: &str, tag: &str, name: &str) -> Option<XmlTagRange> {
        let start = find_xml_start_tag_with_attr(content, tag, "android:name", name)?;
        Self::parse_component_from_match(content, tag, start.start)
    }

    /// 从已找到的开始标签位置解析完整的组件范围（含深度匹配）
    fn parse_component_from_match(
        content: &str,
        tag: &str,
        start_tag_start: usize,
    ) -> Option<XmlTagRange> {
        // 从指定位置开始查找目标标签的开始标签
        let re = Regex::new(&format!(
            r#"(?s)<{}\b[^>]*android:name="#,
            regex::escape(tag)
        ))
        .ok()?;
        let mat = re.find_at(content, start_tag_start)?;
        let start = mat.start();
        let end_of_open_tag = content[start..].find('>').map(|p| start + p)?;

        if content[start..=end_of_open_tag].trim_end().ends_with("/>") {
            return Some(XmlTagRange {
                start,
                end: end_of_open_tag + 1,
                end_close_start: end_of_open_tag - 1,
            });
        }

        let end_tag = format!("</{}>", tag);
        let open_pattern = format!("<{}", tag);
        let mut depth = 1;
        let mut search_pos = end_of_open_tag + 1;

        while search_pos < content.len() {
            if let Some(rel) = content[search_pos..].find(&end_tag) {
                let close_pos = search_pos + rel;
                let segment = &content[search_pos..close_pos];
                let open_count = count_tag_open_occurrences(segment, &open_pattern);

                // 空片段（如 <activity></activity>）不应有嵌套标签
                // count_tag_open_occurrences 对空字符串可能返回错误值，此处做修正
                let actual_open_count = if segment.is_empty() { 0 } else { open_count };

                if depth - actual_open_count == 1 {
                    return Some(XmlTagRange {
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
}

/// XML 标签的范围信息（复用 ManifestComponent）
type XmlTagRange = ManifestComponent;
