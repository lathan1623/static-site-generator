pub const HEADER: &str = r#"
<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
    </head>
"#;

pub const FOOTER: &str = r#"
</html>
"#;

pub fn create_body(body: &str) -> String {
    format!(
        r#"
        <body>
            {}
        </body>
        "#,
        body
    )
}
