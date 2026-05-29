#![allow(dead_code)]
use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::path::Path;

pub struct XmlDocument {
    content: String,
}

impl XmlDocument {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self { content })
    }

    pub fn from_str(content: &str) -> Result<Self> {
        Ok(Self {
            content: content.to_string(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.content
    }

    pub fn to_string(&self) -> String {
        self.content.clone()
    }
}

pub fn read_xml_file(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

fn name_str(n: impl AsRef<[u8]>) -> String {
    String::from_utf8_lossy(n.as_ref()).to_string()
}

pub fn parse_xml_attributes(content: &str, xpath: &str) -> Result<HashMap<String, String>> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);
    let mut attrs = HashMap::new();
    let target_tag = xpath.split('/').last().unwrap_or(xpath);
    let target_bytes = target_tag.as_bytes();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                if e.local_name().as_ref() == target_bytes {
                    for attr_result in e.attributes() {
                        if let Ok(attr) = attr_result {
                            let key = name_str(attr.key);
                            let value = attr
                                .unescape_value()
                                .map(|v| v.into_owned())
                                .unwrap_or_default();
                            attrs.insert(key, value);
                        }
                    }
                    break;
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(attrs)
}

pub fn set_xml_attribute(
    content: &str,
    xpath: &str,
    attr_name: &str,
    attr_value: &str,
) -> Result<String> {
    let target_tag = xpath.split('/').last().unwrap_or(xpath);
    let target_bytes = target_tag.as_bytes();
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);
    let mut output = String::new();
    let mut buf = Vec::new();
    let mut found = false;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) if e.local_name().as_ref() == target_bytes => {
                found = true;
                let mut new_attrs: Vec<(String, String)> =
                    match e.attributes().collect::<Result<Vec<_>, _>>() {
                        Ok(attrs) => attrs
                            .into_iter()
                            .map(|a| {
                                let key = name_str(a.key);
                                let value = a
                                    .unescape_value()
                                    .map(|v| v.into_owned())
                                    .unwrap_or_default();
                                Ok((key, value))
                            })
                            .collect::<Result<Vec<_>>>()?,
                        Err(_) => Vec::new(),
                    };

                let idx = new_attrs.iter().position(|(k, _)| k == attr_name);
                match idx {
                    Some(i) => new_attrs[i].1 = attr_value.to_string(),
                    None => new_attrs.push((attr_name.to_string(), attr_value.to_string())),
                }

                output.push('<');
                output.push_str(&name_str(e.local_name()));
                for (k, v) in &new_attrs {
                    output.push(' ');
                    output.push_str(k);
                    output.push_str("=\"");
                    output.push_str(&escape_xml_attr(v));
                    output.push('"');
                }
                output.push('>');
            }
            Event::Start(e) => {
                output.push('<');
                output.push_str(&name_str(e.name()));
                for attr_result in e.attributes() {
                    if let Ok(attr) = attr_result {
                        output.push(' ');
                        output.push_str(&name_str(attr.key));
                        output.push_str("=\"");
                        output.push_str(
                            &attr
                                .unescape_value()
                                .map(|v| v.into_owned())
                                .unwrap_or_default(),
                        );
                        output.push('"');
                    }
                }
                output.push('>');
            }
            Event::Empty(ref e) if e.local_name().as_ref() == target_bytes => {
                found = true;
                let mut new_attrs: Vec<(String, String)> =
                    match e.attributes().collect::<Result<Vec<_>, _>>() {
                        Ok(attrs) => attrs
                            .into_iter()
                            .map(|a| {
                                let key = name_str(a.key);
                                let value = a
                                    .unescape_value()
                                    .map(|v| v.into_owned())
                                    .unwrap_or_default();
                                Ok((key, value))
                            })
                            .collect::<Result<Vec<_>>>()?,
                        Err(_) => Vec::new(),
                    };

                let idx = new_attrs.iter().position(|(k, _)| k == attr_name);
                match idx {
                    Some(i) => new_attrs[i].1 = attr_value.to_string(),
                    None => new_attrs.push((attr_name.to_string(), attr_value.to_string())),
                }

                output.push('<');
                output.push_str(&name_str(e.local_name()));
                for (k, v) in &new_attrs {
                    output.push(' ');
                    output.push_str(k);
                    output.push_str("=\"");
                    output.push_str(&escape_xml_attr(v));
                    output.push('"');
                }
                output.push_str("/>");
            }
            Event::Empty(e) => {
                output.push('<');
                output.push_str(&name_str(e.name()));
                for attr_result in e.attributes() {
                    if let Ok(attr) = attr_result {
                        output.push(' ');
                        output.push_str(&name_str(attr.key));
                        output.push_str("=\"");
                        output.push_str(
                            &attr
                                .unescape_value()
                                .map(|v| v.into_owned())
                                .unwrap_or_default(),
                        );
                        output.push('"');
                    }
                }
                output.push_str("/>");
            }
            Event::Text(e) => {
                output.push_str(&text_decode(e)?);
            }
            Event::End(e) => {
                output.push_str("</");
                output.push_str(&name_str(e.name()));
                output.push('>');
            }
            Event::CData(e) => {
                output.push_str("<![CDATA[");
                output.push_str(&String::from_utf8_lossy(e.as_ref()));
                output.push_str("]]>");
            }
            Event::Comment(e) => {
                output.push_str("<!--");
                output.push_str(&String::from_utf8_lossy(e.as_ref()));
                output.push_str("-->");
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    if !found {
        return Err(anyhow::anyhow!("Tag '{}' not found in XML", target_tag));
    }
    Ok(output)
}

pub fn get_xml_text_content(content: &str, xpath: &str) -> Result<Option<String>> {
    let target_tag = xpath.split('/').last().unwrap_or(xpath);
    let target_bytes = target_tag.as_bytes();
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) if e.local_name().as_ref() == target_bytes => {
                if let Event::Text(text) = reader.read_event_into(&mut Vec::new())? {
                    return Ok(Some(text_decode(text)?));
                }
                return Ok(None);
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(None)
}

pub fn set_xml_text_content(content: &str, xpath: &str, text_value: &str) -> Result<String> {
    let target_tag = xpath.split('/').last().unwrap_or(xpath);
    let target_bytes = target_tag.as_bytes();
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);
    let mut output = String::new();
    let mut buf = Vec::new();
    let mut inside_target = false;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) if e.local_name().as_ref() == target_bytes => {
                inside_target = true;
                output.push('<');
                output.push_str(&name_str(e.name()));
                for attr_result in e.attributes() {
                    if let Ok(attr) = attr_result {
                        output.push(' ');
                        output.push_str(&name_str(attr.key));
                        output.push_str("=\"");
                        output.push_str(
                            &attr
                                .unescape_value()
                                .map(|v| v.into_owned())
                                .unwrap_or_default(),
                        );
                        output.push('"');
                    }
                }
                output.push('>');
            }
            Event::Start(e) => {
                output.push('<');
                output.push_str(&name_str(e.name()));
                for attr_result in e.attributes() {
                    if let Ok(attr) = attr_result {
                        output.push(' ');
                        output.push_str(&name_str(attr.key));
                        output.push_str("=\"");
                        output.push_str(
                            &attr
                                .unescape_value()
                                .map(|v| v.into_owned())
                                .unwrap_or_default(),
                        );
                        output.push('"');
                    }
                }
                output.push('>');
            }
            Event::Text(_) if inside_target => {
                output.push_str(&escape_xml_text(text_value));
                inside_target = false;
            }
            Event::Text(e) => {
                output.push_str(&text_decode(e)?);
            }
            Event::Empty(e) => {
                output.push('<');
                output.push_str(&name_str(e.name()));
                for attr_result in e.attributes() {
                    if let Ok(attr) = attr_result {
                        output.push(' ');
                        output.push_str(&name_str(attr.key));
                        output.push_str("=\"");
                        output.push_str(
                            &attr
                                .unescape_value()
                                .map(|v| v.into_owned())
                                .unwrap_or_default(),
                        );
                        output.push('"');
                    }
                }
                output.push_str("/>");
            }
            Event::End(e) => {
                output.push_str("</");
                output.push_str(&name_str(e.name()));
                output.push('>');
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(output)
}

pub fn render_template(content: &str, variables: &HashMap<&str, &str>) -> Result<String> {
    let mut result = content.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    Ok(result)
}

fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn escape_xml_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn text_decode(e: quick_xml::events::BytesText) -> Result<String> {
    Ok(String::from_utf8_lossy(e.as_ref()).to_string())
}

#[cfg(test)]
mod tests {
    use super::set_xml_attribute;

    #[test]
    fn set_xml_attribute_updates_empty_tag() {
        let xml = r#"<hbuilder><apps><app appid="HelloH5" appver=""/></apps></hbuilder>"#;
        let updated = set_xml_attribute(xml, "/apps/app", "appid", "__UNI__AA97490").unwrap();

        assert!(updated.contains(r#"<app appid="__UNI__AA97490" appver=""/>"#));
        assert!(!updated.contains("HelloH5"));
    }
}
