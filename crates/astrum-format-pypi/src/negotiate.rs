use crate::json;

pub const HTML_V1: &str = "application/vnd.pypi.simple.v1+html";
pub const TEXT_HTML: &str = "text/html";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wanted {
    Json,
    Html(&'static str),
}

impl Wanted {
    pub fn content_type(self) -> &'static str {
        match self {
            Wanted::Json => json::CONTENT_TYPE,
            Wanted::Html(HTML_V1) => HTML_V1,
            Wanted::Html(_) => "text/html; charset=utf-8",
        }
    }
}

fn candidate(media: &str) -> Option<(Wanted, u8)> {
    match media {
        json::CONTENT_TYPE | "application/vnd.pypi.simple.latest+json" => Some((Wanted::Json, 3)),
        HTML_V1 | "application/vnd.pypi.simple.latest+html" => Some((Wanted::Html(HTML_V1), 3)),
        TEXT_HTML => Some((Wanted::Html(TEXT_HTML), 2)),
        "text/*" | "*/*" => Some((Wanted::Html(TEXT_HTML), 1)),
        "application/*" => Some((Wanted::Json, 1)),
        _ => None,
    }
}

pub fn choose(accept: Option<&str>, format: Option<&str>) -> Wanted {
    if let Some(format) = format.and_then(|value| candidate(value.trim())) {
        return format.0;
    }
    let Some(accept) = accept.filter(|value| !value.trim().is_empty()) else {
        return Wanted::Html(TEXT_HTML);
    };
    let mut best: Option<(f32, u8, Wanted)> = None;
    for range in accept.split(',') {
        let mut parts = range.split(';');
        let media = parts.next().unwrap_or_default().trim().to_ascii_lowercase();
        let quality = parts
            .filter_map(|parameter| parameter.split_once('='))
            .find(|(name, _)| name.trim().eq_ignore_ascii_case("q"))
            .and_then(|(_, value)| value.trim().parse::<f32>().ok())
            .unwrap_or(1.0);
        let Some((wanted, specificity)) = candidate(&media) else {
            continue;
        };
        if quality <= 0.0 {
            continue;
        }
        let better = match best {
            None => true,
            Some((q, s, _)) => quality > q || (quality == q && specificity > s),
        };
        if better {
            best = Some((quality, specificity, wanted));
        }
    }
    best.map(|(_, _, wanted)| wanted)
        .unwrap_or(Wanted::Html(TEXT_HTML))
}

pub fn format_parameter(query: Option<&str>) -> Option<String> {
    query?
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(name, _)| *name == "format")
        .map(|(_, value)| {
            percent_encoding::percent_decode_str(value)
                .decode_utf8_lossy()
                .into_owned()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pip_gets_json() {
        let pip = "application/vnd.pypi.simple.v1+json, application/vnd.pypi.simple.v1+html; q=0.1, text/html; q=0.01";
        assert_eq!(choose(Some(pip), None), Wanted::Json);
    }

    #[test]
    fn browsers_and_old_clients_get_html() {
        assert_eq!(choose(None, None), Wanted::Html(TEXT_HTML));
        assert_eq!(choose(Some("*/*"), None), Wanted::Html(TEXT_HTML));
        assert_eq!(
            choose(
                Some("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
                None
            ),
            Wanted::Html(TEXT_HTML)
        );
        assert_eq!(
            choose(Some("application/vnd.pypi.simple.v1+html"), None),
            Wanted::Html(HTML_V1)
        );
        assert_eq!(
            choose(Some("application/json"), None),
            Wanted::Html(TEXT_HTML)
        );
        assert_eq!(
            Wanted::Html(TEXT_HTML).content_type(),
            "text/html; charset=utf-8"
        );
    }

    #[test]
    fn quality_decides() {
        assert_eq!(
            choose(
                Some("application/vnd.pypi.simple.v1+json;q=0.2, text/html"),
                None
            ),
            Wanted::Html(TEXT_HTML)
        );
        assert_eq!(
            choose(
                Some("text/html;q=0, application/vnd.pypi.simple.latest+json;q=0.5"),
                None
            ),
            Wanted::Json
        );
    }

    #[test]
    fn the_format_parameter_overrides_accept() {
        let format = format_parameter(Some("a=1&format=application%2Fvnd.pypi.simple.v1%2Bjson"));
        assert_eq!(
            format.as_deref(),
            Some("application/vnd.pypi.simple.v1+json")
        );
        assert_eq!(choose(Some("text/html"), format.as_deref()), Wanted::Json);
        assert_eq!(format_parameter(None), None);
    }
}
