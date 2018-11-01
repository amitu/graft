use comrak::{markdown_to_html, ComrakOptions};
use context::Context;
use failure::{err_msg, Error};
use serde_json;
use serde_yaml;
use std::str;

#[derive(Debug, PartialEq)]
pub enum Exec {
    SQL,
    Shell,
    Python,
}

#[derive(Debug, PartialEq)]
pub enum Format {
    Text,
    Markdown,
    SQL, // table
    YAML,
    JSON,
}

#[derive(Debug)]
pub struct Section {
    pub include: Option<String>,
    pub reference: String,
    pub format: Format,
    pub process: Option<Exec>,
    pub body: serde_json::Value,
}

impl Section {
    pub fn from(header: &str, body: &str) -> Result<Section, Error> {
        let mut section = Section {
            include: None,
            reference: "ROOT".into(),
            format: Format::YAML,
            process: None,
            body: serde_json::Value::Null,
        };

        println!("body: {} <- body", body);
        println!("{} => {:?}", &header, &section);

        for part in header.split_whitespace() {
            if part == "--" {
                continue;
            }

            if part.starts_with("$") {
                section.include = Some(part[1..].into());
                continue;
            }

            if part.starts_with("@") {
                section.reference = part[1..].into();
                continue;
            }

            if part.starts_with("!") {
                section.process = Some(match part.to_lowercase().as_ref() {
                    "!sql" => Exec::SQL,
                    "!sh" | "!shell" => Exec::Shell,
                    "!py" | "!python" => Exec::Python,
                    _ => return Err(err_msg(format!("invalid process: {}", part))),
                });
                continue;
            }

            if part.starts_with("~") {
                section.format = match part.to_lowercase().as_ref() {
                    "~text" => Format::Text,
                    "~md" | "~markdown" => Format::Markdown,
                    "~sql" => Format::SQL,
                    "~yml" | "~yaml" => Format::YAML,
                    "~json" => Format::JSON,
                    _ => return Err(err_msg(format!("invalid format: {}", part))),
                };
                continue;
            }

            return Err(err_msg(format!("invalid input: {}", part)));
        }

        // TODO: handle exec
        section.body = match section.format {
            Format::Text => serde_json::Value::String(body.into()),
            Format::Markdown => {
                serde_json::Value::String(markdown_to_html(body, &ComrakOptions::default()))
            }
            Format::JSON => serde_json::from_str(body)?,
            Format::YAML => serde_yaml::from_str(body)?,
            Format::SQL => {
                serde_json::Value::String(body.into()) // TODO
            }
        };

        println!("{} => {:?}", &header, &section);
        Ok(section)
    }

    pub fn parse(txt: &str) -> Result<Vec<Section>, Error> {
        let txt = "\n".to_owned() + txt;
        println!("txt: {}", &txt);
        let mut sections = vec![];
        for part in txt.split("\n--").skip(1) {
            let part = part.to_owned() + "\n";
            let split = part.splitn(2, '\n').collect::<Vec<&str>>();
            let (header, body) = (split[0], split[1]);
            println!("header: {} ::: body: {}", &header, &body);
            sections.push(Section::from(header, body)?);
        }
        Ok(sections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from() {
        let s = Section::from("-- $include @ROOT !sql", "foo: bar").unwrap();
        assert_eq!(s.include, Some("include".into()));
        assert_eq!(s.reference, "ROOT");
        assert_eq!(s.format, Format::YAML);
        assert_eq!(s.process, Some(Exec::SQL));
        assert_eq!(s.body, json!({"foo": "bar"}));

        let s = Section::from("-- $include ~text", "yo").unwrap();
        assert_eq!(s.include, Some("include".into()));
        assert_eq!(s.reference, "ROOT");
        assert_eq!(s.format, Format::Text);
        assert_eq!(s.process, None);
        assert_eq!(s.body, json!("yo"));
    }
}
