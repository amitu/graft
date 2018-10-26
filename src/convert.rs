use context::Context;
use failure::Error;
use section::Section;
use serde_json;

pub fn convert<T>(txt: &str, context: &T) -> Result<serde_json::Value, Error>
where
    T: Context,
{
    let sections = Section::parse(txt)?;
    let mut value = serde_json::Value::Null;
    for section in sections {
        value = section.apply(value, context)?;
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use assert_snapshot;
    use context::StaticContext;
    use serde_json;
    use textwrap::dedent as d;

    fn t(txt: &str, ctx: &StaticContext, reference: serde_json::Value) {
        assert_eq!(super::convert(&d(txt), ctx).unwrap(), reference);
    }

    #[test]
    fn convert() {
        let ctx = StaticContext::new(
            "foo.json",
            json!({
                "hello": "world",
                "main": {
                    "$ref": "main",
                    "$default": "yo",
                },
                "children": {
                    "$ref": "children",
                    "list": true,
                    "sample": {
                        "name": "something",
                        "value": "some value",
                    },
                }
            }),
        ).with("title.txt", json!("this is the title"));

        t(
            r#"
                -- @ROOT~json
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
            r"-- @ROOT#foo",
            &ctx,
            json!({
                "hello": "world",
                "main": "yo",
                "children": []
            }),
        );

        t(
            r#"
                -- @ROOT#foo
                main: "hello main"
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": "hello main",
                "children": []
            }),
        );

        t(
            r#"
                -- @ROOT#foo
                -- @main~text
                hello main
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": "hello main",
                "children": []
            }),
        );

        t(
            r#"
                -- @ROOT#foo
                -- @main#title~text
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": "this is the title",
                "children": []
            }),
        );
    }
}
