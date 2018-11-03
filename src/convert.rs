use context::Context;
use failure::{err_msg, Error};
use section::Section;
use serde_json;

pub fn convert<T>(txt: &str, ctx: &T) -> Result<serde_json::Value, Error>
where
    T: Context,
{
    println!("text: {}", txt);
    let sections = Section::parse(txt, ctx)?;
    println!("sections: {:?}", &sections);
    eval("ROOT", &sections, 0)
}

fn eval(path: &str, sections: &[Section], start: usize) -> Result<serde_json::Value, Error> {
    for (idx, section) in sections.iter().enumerate() {
        if idx < start {
            continue;
        }
        if section.reference != path {
            continue;
        }
        return digest(&section.body, sections, idx);
    }
    println!(
        "not found '{}' starting at {} in {:?}",
        path, start, sections
    );
    Err(err_msg(format!("not found: {}", path)))
}

fn digest(
    body: &serde_json::Value,
    sections: &[Section],
    start: usize,
) -> Result<serde_json::Value, Error> {
    if let serde_json::Value::Object(o) = body {
        let mut n = serde_json::Map::new();
        for (k, v) in o {
            if !v.is_object() {
                n.insert(k.to_string(), v.clone());
                continue;
            }
            let ov = v.as_object().unwrap(); // safe because we have already checked
            if !ov.contains_key("$ref") {
                n.insert(k.to_string(), v.clone());
                continue;
            }

            let ref_ = ov.get("$ref").unwrap(); // safe because we have already checked
            if !ref_.is_string() {
                return Err(err_msg(format!("$ref if not a string: {:?}", &ref_)));
            }

            let ref_ = ref_.as_str().unwrap(); // safe because we have already checked

            let v = if has_path(k, sections, start + 1) {
                eval(ref_, sections, start + 1)?
            } else {
                ov.get("default")
                    .ok_or_else(|| err_msg(format!("'{}' not found", ref_)))?
                    .clone()
            };
            n.insert(k.to_string(), v);
        }
        return Ok(serde_json::Value::Object(n));
    } else {
        return Ok(body.clone());
    }
}

fn has_path(path: &str, sections: &[Section], start: usize) -> bool {
    for (idx, section) in sections.iter().enumerate() {
        if idx < start {
            continue;
        }
        if section.reference != path {
            continue;
        }
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use context::StaticContext;
    use serde_json;
    use textwrap::dedent as d;

    fn t(txt: &str, ctx: &StaticContext, reference: serde_json::Value) {
        assert_eq!(
            super::convert(&d(txt.trim_right()), ctx).unwrap(),
            reference
        );
    }

    #[test]
    fn convert() {
        let ctx = StaticContext::new(
            "foo.json",
            r#"
            {
                "hello": "world",
                "main": {
                    "$ref": "main",
                    "default": "yo"
                }
            }
            "#,
        ).with("title.txt", "this is the title");

        t(
            r#"
                -- @ROOT ~json
                {
                    "yo": "man"
                }
            "#,
            &ctx,
            json!({
                "yo": "man"
            }),
        );
        t(
            r#"
                -- @ROOT
                yo: man
            "#,
            &ctx,
            json!({
                "yo": "man"
            }),
        );

        t(
            r#"
                -- $foo
                -- @main ~text
                hello main
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": "hello main",
            }),
        );

        t(
            r"-- @ROOT $foo",
            &ctx,
            json!({
                "hello": "world",
                "main": "yo",
            }),
        );

        t(
            r#"
                -- @ROOT $foo
                -- @main $title ~text
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": "this is the title",
            }),
        );

        /*
        t(
            r#"
                -- @ROOT #foo
                main: "hello main"
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": "hello main",
                "children": []
            }),
        );

        */
    }
}
