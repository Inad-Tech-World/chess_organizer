//! HTML output: on-screen preview, printing (via print CSS) and the
//! publishing site all use this one template.

use crate::{Report, Result};
use minijinja::{Environment, context};

const TEMPLATE: &str = include_str!("../templates/list.html");
const CSS: &str = include_str!("../templates/list.css");

pub fn render(report: &Report) -> Result<String> {
    let mut env = Environment::new();
    env.add_template("list.html", TEMPLATE)?;
    let html = env
        .get_template("list.html")?
        .render(context! { r => report, css => CSS })?;
    Ok(html)
}
