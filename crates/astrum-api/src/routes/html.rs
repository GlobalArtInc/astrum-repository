use axum::http::{HeaderMap, Method, header};

const STYLE: &str = "\
:root{color-scheme:light dark;--bg:#fcfcfd;--panel:#f7f7f9;--line:#dbdbe2;--text:#17171c;--muted:#5f6170;--link:#4434c4;--hover:#f0f0f3}\
@media (prefers-color-scheme:dark){:root{--bg:#0e1015;--panel:#151821;--line:#2b303d;--text:#e7e9ef;--muted:#9aa2b3;--link:#cfc7ff;--hover:#1a1d27}}\
*{box-sizing:border-box}\
body{margin:0;background:var(--bg);color:var(--text);font:14px/1.5 \"Geist\",system-ui,-apple-system,\"Segoe UI\",sans-serif}\
header{border-bottom:1px solid var(--line);background:var(--panel);padding:12px 24px;font-weight:600}\
header a{color:var(--text);text-decoration:none}\
main{max-width:1100px;margin:0 auto;padding:24px}\
h1{font-size:20px;font-weight:600;margin:0 0 8px;overflow-wrap:anywhere}\
p{color:var(--muted);margin:0 0 16px}\
a{color:var(--link)}\
table{width:100%;border-collapse:collapse;border:1px solid var(--line);border-radius:8px;overflow:hidden}\
th,td{padding:8px 12px;text-align:left;border-bottom:1px solid var(--line)}\
th{background:var(--panel);color:var(--muted);font-weight:500}\
tr:last-child td{border-bottom:0}\
tbody tr:hover{background:var(--hover)}\
td a{text-decoration:none;overflow-wrap:anywhere}\
td+td,th+th{text-align:right;white-space:nowrap;color:var(--muted)}";

pub(crate) fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(crate) fn page(title: &str, body: &str) -> String {
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{title}</title>\n<style>{STYLE}</style>\n</head>\n<body>\n<header><a href=\"/\">Astrum Repository</a></header>\n<main>\n<h1>{title}</h1>\n{body}</main>\n</body>\n</html>\n"
    )
}

pub(crate) fn browser_navigation(method: &Method, headers: &HeaderMap) -> bool {
    let value = |name: &str| {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase()
    };
    method == Method::GET
        && value(header::ACCEPT.as_str()).contains("text/html")
        && (value("sec-fetch-mode") == "navigate"
            || value("sec-fetch-dest") == "document"
            || value("upgrade-insecure-requests") == "1")
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(*name, HeaderValue::from_static(value));
        }
        map
    }

    #[test]
    fn only_a_browser_opening_a_page_is_a_navigation() {
        let browser = headers(&[
            ("accept", "text/html,application/xhtml+xml,*/*;q=0.8"),
            ("sec-fetch-mode", "navigate"),
        ]);
        assert!(browser_navigation(&Method::GET, &browser));
        assert!(!browser_navigation(&Method::HEAD, &browser));
        let older = headers(&[("accept", "text/html"), ("upgrade-insecure-requests", "1")]);
        assert!(browser_navigation(&Method::GET, &older));
        let tool = headers(&[("accept", "text/html, image/gif, image/jpeg, *; q=.2")]);
        assert!(!browser_navigation(&Method::GET, &tool));
        let fetch = headers(&[("accept", "application/json"), ("sec-fetch-mode", "cors")]);
        assert!(!browser_navigation(&Method::GET, &fetch));
    }

    #[test]
    fn pages_escape_nothing_themselves_but_offer_escape() {
        assert_eq!(
            escape("<a href=\"x\">&</a>"),
            "&lt;a href=&quot;x&quot;&gt;&amp;&lt;/a&gt;"
        );
        let html = page("Index of /x/", "<p>body</p>\n");
        assert!(html.contains("<title>Index of /x/</title>"));
        assert!(html.contains("prefers-color-scheme:dark"));
    }
}
