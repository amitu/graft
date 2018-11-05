use context::Context;
use failure::{err_msg, Error};
use section::Section;
use serde_json;

pub fn convert<T>(txt: &str, ctx: &T) -> Result<serde_json::Value, Error>
where
    T: Context,
{
    let sections = Section::parse(txt, ctx)?;
    eval("ROOT", &sections, 0, "END")
}

fn eval(
    path: &str,
    sections: &[Section],
    start: usize,
    till: &str,
) -> Result<serde_json::Value, Error> {
    for (idx, section) in sections.iter().enumerate() {
        if section.reference == till {
            break;
        }
        if idx < start {
            continue;
        }
        if section.reference != path {
            continue;
        }
        let prefix = if path == "ROOT" {
            "".to_string()
        } else {
            path.to_string() + "/"
        };
        return digest(&section.body, sections, idx, prefix, till);
    }
    Err(err_msg(format!("not found: {}", path)))
}

fn eval_list(
    path: &str,
    sections: &[Section],
    start: usize,
    till: &str,
) -> Result<serde_json::Value, Error> {
    let mut lst = vec![];

    for (idx, section) in sections.iter().enumerate() {
        if section.reference == till {
            break;
        }
        if idx < start {
            continue;
        }
        if section.reference != path {
            continue;
        }
        let prefix = if path == "ROOT" {
            "".to_string()
        } else {
            path.to_string() + "/"
        };
        lst.push(digest(&section.body, sections, idx, prefix, till)?);
    }

    Ok(serde_json::Value::Array(lst))
}

fn digest(
    body: &serde_json::Value,
    sections: &[Section],
    start: usize,
    prefix: String,
    till: &str,
) -> Result<serde_json::Value, Error> {
    if let serde_json::Value::Object(o) = body {
        let mut n = serde_json::Map::new();
        for (k, v) in o {
            if !v.is_object() {
                n.insert(
                    k.to_string(),
                    digest(v, sections, start, prefix.clone(), till)?,
                );
                continue;
            }
            let ov = v.as_object().unwrap(); // safe because we have already checked
            if !ov.contains_key("$ref") {
                n.insert(
                    k.to_string(),
                    digest(v, sections, start, prefix.clone(), till)?,
                );
                continue;
            }

            let ref_ = ov.get("$ref").unwrap(); // safe because we have already checked
            if !ref_.is_string() {
                return Err(err_msg(format!("$ref if not a string: {:?}", &ref_)));
            }

            let ref_ = prefix.clone() + ref_.as_str().unwrap(); // safe because we have already checked

            let v = if has_path(&ref_, sections, start + 1, till) {
                if ref_.ends_with("[]") {
                    eval_list(&ref_, sections, start + 1, till)?
                } else {
                    eval(&ref_, sections, start + 1, till)?
                }
            } else {
                ov.get("default")
                    .ok_or_else(|| err_msg(format!("'{}' not found", ref_)))?
                    .clone()
            };
            n.insert(k.to_string(), v);
        }
        return Ok(serde_json::Value::Object(n));
    } else if let serde_json::Value::Array(a) = body {
        let mut n = vec![];
        for item in a {
            n.push(digest(item, sections, start, prefix.clone(), till)?);
        }
        return Ok(serde_json::Value::Array(n));
    } else {
        return Ok(body.clone());
    }
}

fn has_path(path: &str, sections: &[Section], start: usize, till: &str) -> bool {
    for (idx, section) in sections.iter().enumerate() {
        if section.reference == till {
            break;
        }
        if idx < start {
            continue;
        }
        if section.reference != path {
            continue;
        }
        return true;
    }
    path.ends_with("[]")
}

#[cfg(test)]
mod tests {
    use context::StaticContext;
    use serde_json;
    use textwrap::dedent as d;

    fn t(txt: &str, ctx: &StaticContext, reference: serde_json::Value) {
        println!("============= TEST ==============");
        println!("{}", txt);
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
                },
                "obj": {
                    "list": {
                        "$ref": "children[]"
                    }
                },
                "main2": {
                    "$ref": "main2",
                    "default": "yo2"
                }
            }
            "#,
        ).with("title.txt", "this is the title")
        .with(
            "bar.json",
            r#"{
                "bar": {
                    "$ref": "bar"
                }
            }"#,
        ).with("foo2.graft", "-- $foo\n-- @main $bar\n")
        .with(
            "nested.json",
            r#"{
                "title": {
                    "$ref": "title"
                },
                "obj": {
                    "title": {
                        "$ref": "title"
                    }
                }
            }"#,
        ).with(
            "array.json",
            r#"{
                "title": {
                    "$ref": "title"
                },
                "obj": [{
                    "title": {
                        "$ref": "title"
                    }
                }]
            }"#,
        );

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
                "obj": {
                    "list": []
                },
                "main2": "yo2",
            }),
        );

        t(
            r#"
                -- $foo
                main: hello main
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": "hello main",
                "obj": {
                    "list": []
                },
                "main2": "yo2",
            }),
        );

        t(
            r#"
                -- @ROOT $foo
                -- @main ~text
                hello main
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": "hello main",
                "obj": {
                    "list": []
                },
                "main2": "yo2",
            }),
        );

        t(
            r"-- @ROOT $foo",
            &ctx,
            json!({
                "hello": "world",
                "main": "yo",
                "obj": {
                    "list": []
                },
                "main2": "yo2",
            }),
        );

        t(
            r#"
                -- $foo
                -- @main $title ~text
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": "this is the title",
                "obj": {
                    "list": []
                },
                "main2": "yo2",
            }),
        );

        t(
            r#"
                -- $foo
                -- @main $bar
                -- @main/bar
                x: 20
                y: 10
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": {
                    "bar": {
                        "y": 10,
                        "x": 20
                    }
                },
                "obj": {
                    "list": []
                },
                "main2": "yo2",
            }),
        );

        t(
            r#"
                -- $foo2
                -- @main/bar
                x: 20
                y: 10
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": {
                    "bar": {
                        "y": 10,
                        "x": 20
                    }
                },
                "obj": {
                    "list": []
                },
                "main2": "yo2",
            }),
        );

        t(
            r#"
                -- $foo2
                main2: la la la
                -- @main/bar
                x: 20
                y: 10
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": {
                    "bar": {
                        "y": 10,
                        "x": 20
                    }
                },
                "obj": {
                    "list": []
                },
                "main2": "la la la",
            }),
        );

        t(
            r#"
                -- $foo
                -- @main $bar
                -- @main/bar
                x: 20
                y: 10
                -- @main2 $bar
                -- @main2/bar
                a: 20
                b: 10
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": {
                    "bar": {
                        "y": 10,
                        "x": 20
                    }
                },
                "obj": {
                    "list": []
                },
                "main2": {
                    "bar": {
                        "b": 10,
                        "a": 20
                    }
                },
            }),
        );

        t(
            r#"
                -- $foo
                -- @main $title ~text
                -- @children[] ~text
                child 1
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": "this is the title",
                "obj": {
                    "list": ["child 1"]
                },
                "main2": "yo2",
            }),
        );

        t(
            r#"
                -- $foo
                -- @main $title ~text
                -- @children[] ~text
                child 1
                -- @children[] ~text
                child 2
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": "this is the title",
                "obj": {
                    "list": ["child 1", "child 2"]
                },
                "main2": "yo2",
            }),
        );

        t(
            r#"
                -- $foo
                -- @main $title ~text
                -- @children[] $bar
                -- @children[]/bar
                a: 20
                b: 10
                -- @children[] $bar
                -- @children[]/bar
                x: 20
                y: 10
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": "this is the title",
                "obj": {
                    "list": [
                        {
                            "bar": {
                                "b": 10,
                                "a": 20
                            }
                        },
                        {
                            "bar": {
                                "y": 10,
                                "x": 20
                            }
                        },
                    ]
                },
                "main2": "yo2",
            }),
        );

        t(
            r#"
                -- $foo
                -- @main $title ~text
                -- @children[] $bar
                -- @children[]/bar
                a: 20
                b: 10
                -- @children[] $foo
                -- @children[]/children[] ~text
                nested child
                -- @children[]/main ~text
                inner main
            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": "this is the title",
                "obj": {
                    "list": [
                        {
                            "bar": {
                                "b": 10,
                                "a": 20
                            }
                        },
                        {
                            "hello": "world",
                            "main": "inner main",
                            "obj": {
                                "list": ["nested child"]
                            },
                            "main2": "yo2",
                        },
                    ]
                },
                "main2": "yo2",
            }),
        );

        t(
            r#"
                -- $nested
                -- @title ~text
                the title
            "#,
            &ctx,
            json!({
                "title": "the title",
                "obj": {
                    "title": "the title",
                },
            }),
        );

        t(
            r#"
                -- $array
                -- @title ~text
                the title
            "#,
            &ctx,
            json!({
                "title": "the title",
                "obj": [{
                    "title": "the title",
                }],
            }),
        );

        /*
        t(
            r#"
                -- $post
                -- @body[] ~md
                # hello world

                this is the cool i am working
                -- @body[] $slideshow
                -- @body[]/slides[] ~md
                slide 1
                -- @body[]/slides[] ~md
                slide 2

                -- @body[] ~md

                end of slide show.

                thanks for.

                -- @body[] $slideshow

            "#,
            &ctx,
            json!({
                "hello": "world",
                "main": "this is the title",
            }),
        );
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
