//! Terse-prose transform: drop filler, greetings, and (at higher levels)
//! articles, while leaving every code block byte-for-byte intact. Levels:
//! off | lite | full | ultra.

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref FILLER: Regex = Regex::new(
        r"(?i)\b(just|really|basically|actually|simply|essentially|very|quite|in order to|please note that|it is worth noting that|i'?d be happy to|i would be happy to)\b"
    ).unwrap();
    static ref GREETING: Regex = Regex::new(
        r"(?i)^\s*(sure|certainly|absolutely|great|gotcha|no problem|happy to help|of course)[!.,]*\s*"
    ).unwrap();
    static ref ARTICLES: Regex = Regex::new(r"(?i)\b(a|an|the)\s+").unwrap();
    static ref MULTISPACE: Regex = Regex::new(r"[ \t]{2,}").unwrap();
    static ref CODE_FENCE: Regex = Regex::new(r"(?s)```.*?```").unwrap();
}

pub fn terse(text: &str, level: &str) -> String {
    if level == "off" {
        return text.to_string();
    }
    let mut blocks: Vec<String> = Vec::new();
    let protected = CODE_FENCE.replace_all(text, |c: &regex::Captures| {
        blocks.push(c[0].to_string());
        format!("\u{0}C{}\u{0}", blocks.len() - 1)
    });
    let mut out = GREETING.replace(&protected, "").into_owned();
    out = FILLER.replace_all(&out, "").into_owned();
    if matches!(level, "full" | "ultra") {
        out = ARTICLES.replace_all(&out, "").into_owned();
    }
    out = MULTISPACE.replace_all(&out, " ").into_owned();
    for (i, b) in blocks.iter().enumerate() {
        out = out.replace(&format!("\u{0}C{i}\u{0}"), b);
    }
    out
}
