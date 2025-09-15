//! Helper crate to create an index containing a list of [link reference definitions](https://spec.commonmark.org/0.31.2/#link-reference-definitions) to all elements in the crate documentation.

#![forbid(unsafe_code)]

use anyhow::{Context, Result};

#[derive(Debug)]
struct Link {
    crate_: String,
    href: String,
    text: Option<String>,
    kind: Option<String>,
    prefix: String,
}

impl Link {
    fn kind(&self) -> String {
        self.kind
            .clone()
            .unwrap_or(href_to_kind(&self.href).to_owned())
    }

    fn as_markdown(&self) -> String {
        let (kind, crate_, text, href, prefix) = (
            self.kind(),
            &self.crate_,
            &self.text,
            &self.href,
            &self.prefix,
        );
        let suffix = text
            .clone()
            .map_or("".to_owned(), |text| format!("::{text}"));
        format!(r#"[`{kind}@{crate_}{suffix}`]: {prefix}{crate_}/{href} "{kind} {crate_}{suffix}""#)
    }
}

/// ```
/// assert_eq!(href_to_kind("type.Result.html"), "type");
/// assert_eq!(href_to_kind("task/fn.block_on_future.html"), "fn");
/// ```
fn href_to_kind(href: &str) -> &str {
    let (kind, _) = href.split_once(".").unwrap();
    match kind.contains("/") {
        true => kind.rsplit_once("/").unwrap().1,
        false => kind,
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("Expected one argument; the prefix of rustdoc links");
    }
    let rustdoc_link_prefix = args.get(1).unwrap();

    for path in std::fs::read_dir("target/doc/")?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|d| d.path())
    {
        let all_html_path = path.join("all.html");
        if all_html_path.exists() {
            let crate_ = path
                .file_name()
                .unwrap()
                .to_str()
                .context("invalid unicode")?
                .to_owned();
            println!(
                "{}",
                Link {
                    crate_: crate_.clone(),
                    href: "index.html".to_owned(),
                    text: None,
                    kind: Some("crate".to_owned()),
                    prefix: rustdoc_link_prefix.to_owned(),
                }
                .as_markdown()
            );
            let all_html = scraper::Html::parse_document(&std::fs::read_to_string(&all_html_path)?);
            for link in all_html.select(&scraper::Selector::parse("main a").unwrap()) {
                let href = link.attr("href").context("link without href")?;
                assert!(href.contains("."));
                println!(
                    "{}",
                    Link {
                        crate_: crate_.clone(),
                        href: href.to_owned(),
                        text: Some(link.inner_html()),
                        kind: None,
                        prefix: rustdoc_link_prefix.to_owned(),
                    }
                    .as_markdown(),
                );
            }

            for entry in walkdir::WalkDir::new(&path) {
                let entry = entry?;
                if !entry.file_type().is_file() {
                    continue;
                };
                if entry.file_name() == "index.html" {
                    let index_html =
                        scraper::Html::parse_document(&std::fs::read_to_string(entry.path())?);

                    // This is similar to how rustdoc's "copy path" button gets the path, so if something changes in a
                    // new rustdoc version that could be used as inspiration to update this.
                    // Since this is only looking for `mod`s, it skips the ` in ` handling.
                    let Some(title) = index_html
                        .select(&scraper::Selector::parse("title").unwrap())
                        .next()
                    else {
                        continue;
                    };
                    let title = String::from_iter(title.text());
                    let Some(title) = title.strip_suffix(" - Rust") else {
                        continue;
                    };
                    let Some((crate_, text)) = title.split_once("::") else {
                        continue;
                    };
                    let Ok(href) = entry.path().strip_prefix(&path) else {
                        continue;
                    };

                    println!(
                        "{}",
                        Link {
                            crate_: crate_.to_owned(),
                            href: href.to_str().unwrap().to_owned(),
                            text: Some(text.to_owned()),
                            kind: Some("mod".to_owned()),
                            prefix: rustdoc_link_prefix.to_owned(),
                        }
                        .as_markdown(),
                    );
                }
            }
        }
    }

    Ok(())
}
