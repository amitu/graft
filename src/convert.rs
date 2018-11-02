use context::Context;
use failure::{err_msg, Error};
use section::Section;
use serde_json;

pub fn convert<T>(txt: &str, ctx: &T) -> Result<serde_json::Value, Error>
where
    T: Context,
{
    let sections = Section::parse(txt, ctx)?;
    eval("ROOT", sections)
}

fn eval(path: &str, sections: Vec<Section>) -> Result<serde_json::Value, Error> {
    for section in sections {
        if section.reference == path {
            return Ok(section.body);
        }
    }
    Err(err_msg("not found"))
}

#[cfg(test)]
mod tests {
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
            r#"
            {
                "hello": "world",
                "main": {
                    "$ref": "main",
                    "$default": "yo"
                },
                "children": {
                    "$ref": "children",
                    "list": true,
                    "sample": {
                        "name": "something",
                        "value": "some value"
                    }
                }
            }
            "#,
        ).with("title.txt", "this is the title");

        t(
            r#"
                -- @ROOT ~json
                {
                    "yo": "man"
                }"#,
            &ctx,
            json!({
                "yo": "man"
            }),
        );
        t(
            r#"
                -- @ROOT
                yo: man"#,
            &ctx,
            json!({
                "yo": "man"
            }),
        );

        t(
            r"-- @ROOT $foo",
            &ctx,
            json!({
                "hello": "world",
                "main": "yo",
                "children": []
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

        t(
            r#"
                -- @ROOT #foo
                -- @main ~text
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
                -- @ROOT #foo
                -- @main #title ~text
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": "this is the title",
                "children": []
            }),
        );
        */
    }
}
