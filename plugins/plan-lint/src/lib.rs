use extism_pdk::*;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Renders a CriticMarkup review block into HTML. The five standard marks map to semantic elements:
/// `{++ins++}`→`<ins>`, `{--del--}`→`<del>`, `{~~a~>b~~}`→`<del>a</del><ins>b</ins>`,
/// `{==mark==}`→`<mark>`, `{>>note<<}`→`<span class="cm-comment">`. Everything else is escaped plain
/// text; the host wraps the result in DOMPurify, which keeps exactly these tags.
fn render_critic(input: &str) -> String {
    let bytes = input.as_bytes();
    let n = input.len();
    let mut out = String::new();
    let mut plain = String::new();
    let mut i = 0;
    while i < n {
        if bytes[i] == b'{' && i + 2 < n {
            let (close, tag): (&str, u8) = match (bytes[i + 1], bytes[i + 2]) {
                (b'+', b'+') => ("++}", 1),
                (b'-', b'-') => ("--}", 2),
                (b'=', b'=') => ("==}", 3),
                (b'~', b'~') => ("~~}", 4),
                (b'>', b'>') => ("<<}", 5),
                _ => ("", 0),
            };
            if tag != 0 {
                if let Some(rel) = input[i + 3..].find(close) {
                    let inner = &input[i + 3..i + 3 + rel];
                    if !plain.is_empty() {
                        out.push_str(&esc(&plain));
                        plain.clear();
                    }
                    match tag {
                        1 => out.push_str(&format!("<ins>{}</ins>", esc(inner))),
                        2 => out.push_str(&format!("<del>{}</del>", esc(inner))),
                        3 => out.push_str(&format!("<mark>{}</mark>", esc(inner))),
                        4 => {
                            let (old, new) = inner.split_once("~>").unwrap_or((inner, ""));
                            out.push_str(&format!(
                                "<del>{}</del><ins>{}</ins>",
                                esc(old),
                                esc(new)
                            ));
                        }
                        _ => out
                            .push_str(&format!("<span class=\"cm-comment\">{}</span>", esc(inner))),
                    }
                    i += 3 + rel + close.len();
                    continue;
                }
            }
        }
        let ch_len = input[i..].chars().next().map_or(1, char::len_utf8);
        plain.push_str(&input[i..i + ch_len]);
        i += ch_len;
    }
    if !plain.is_empty() {
        out.push_str(&esc(&plain));
    }
    format!("<pre class=\"criticmarkup\">{out}</pre>")
}

#[plugin_fn]
pub fn render(input: String) -> FnResult<String> {
    Ok(render_critic(&input))
}

#[cfg(test)]
mod tests {
    use super::render_critic;

    #[test]
    fn renders_all_five_marks() {
        let out = render_critic("keep {++add++} {--drop--} {~~old~>new~~} {==mark==} {>>note<<}");
        assert!(out.contains("<ins>add</ins>"));
        assert!(out.contains("<del>drop</del>"));
        assert!(out.contains("<del>old</del><ins>new</ins>"));
        assert!(out.contains("<mark>mark</mark>"));
        assert!(out.contains("<span class=\"cm-comment\">note</span>"));
    }

    #[test]
    fn escapes_plain_and_inner_text() {
        let out = render_critic("a < b {++x & y++}");
        assert!(out.contains("a &lt; b"));
        assert!(out.contains("<ins>x &amp; y</ins>"));
    }

    #[test]
    fn leaves_unterminated_marks_as_escaped_text() {
        let out = render_critic("{++ never closed");
        assert!(out.contains("{++ never closed"));
    }
}
