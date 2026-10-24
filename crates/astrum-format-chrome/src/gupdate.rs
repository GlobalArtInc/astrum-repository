use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use url::form_urlencoded;

use crate::crx::{valid_id, valid_version};

pub const NAMESPACE: &str = "http://www.google.com/update2/response";
pub const MAX_ENTRIES: usize = 1000;
const MAX_PRODVERSION: usize = 32;
const MAX_DEPTH: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    UpdateCheck,
    Redirect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wanted {
    pub id: String,
    pub version: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub mode: Mode,
    pub prodversion: Option<String>,
    pub wanted: Vec<Wanted>,
}

pub fn is_update_request(query: Option<&str>) -> bool {
    form_urlencoded::parse(query.unwrap_or_default().as_bytes())
        .any(|(name, _)| name == "x" || name == "response")
}

fn valid_prodversion(version: &str) -> bool {
    version.len() <= MAX_PRODVERSION
        && version.split('.').count() <= 4
        && version
            .split('.')
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

pub fn parse_request(query: Option<&str>) -> Result<Request, String> {
    let mut mode = Mode::UpdateCheck;
    let mut prodversion = None;
    let mut wanted: Vec<Wanted> = Vec::new();
    let mut named = 0usize;
    for (name, value) in form_urlencoded::parse(query.unwrap_or_default().as_bytes()) {
        match name.as_ref() {
            "response" => {
                mode = match value.as_ref() {
                    "updatecheck" | "" => Mode::UpdateCheck,
                    "redirect" => Mode::Redirect,
                    other => {
                        return Err(format!(
                            "response={other} is not served; use updatecheck or redirect"
                        ));
                    }
                }
            }
            "prodversion" => {
                prodversion =
                    Some(value.trim().to_owned()).filter(|found| valid_prodversion(found));
            }
            "x" => {
                named += 1;
                if named > MAX_ENTRIES {
                    return Err(format!(
                        "a request may name at most {MAX_ENTRIES} extensions"
                    ));
                }
                let mut id = None;
                let mut version = None;
                for (field, content) in form_urlencoded::parse(value.as_bytes()) {
                    match field.as_ref() {
                        "id" => id = Some(content.trim().to_ascii_lowercase()),
                        "v" => {
                            version =
                                Some(content.trim().to_owned()).filter(|found| valid_version(found))
                        }
                        _ => {}
                    }
                }
                if let Some(id) = id.filter(|id| valid_id(id))
                    && !wanted.iter().any(|known| known.id == id)
                {
                    wanted.push(Wanted { id, version });
                }
            }
            _ => {}
        }
    }
    if wanted.is_empty() {
        return Err(
            "the request names no extension; add x=id%3D<extension id>%26uc for each one"
                .to_owned(),
        );
    }
    Ok(Request {
        mode,
        prodversion,
        wanted,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    Update {
        codebase: String,
        version: String,
        sha256: Option<String>,
        size: Option<u64>,
    },
    NoUpdate,
    Unknown,
}

pub fn escape(text: &str) -> String {
    text.chars()
        .filter(|c| matches!(c, '\t' | '\n' | '\r') || *c >= ' ')
        .collect::<String>()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn daystart(now: DateTime<Utc>) -> (i64, i64) {
    let epoch = NaiveDate::from_ymd_opt(2007, 1, 1).expect("the epoch of the protocol is a date");
    let date = now.date_naive();
    let seconds = now
        .time()
        .signed_duration_since(chrono::NaiveTime::MIN)
        .num_seconds();
    (seconds, (date - epoch).num_days())
}

pub fn document(answers: &[(String, Answer)], now: DateTime<Utc>) -> String {
    let (seconds, days) = daystart(now);
    let mut out = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<gupdate xmlns=\"{NAMESPACE}\" protocol=\"2.0\" server=\"prod\">\n  <daystart elapsed_seconds=\"{seconds}\" elapsed_days=\"{days}\"/>\n"
    );
    for (id, answer) in answers {
        let id = escape(id);
        match answer {
            Answer::Update {
                codebase,
                version,
                sha256,
                size,
            } => {
                out.push_str(&format!(
                    "  <app appid=\"{id}\" cohort=\"\" cohortname=\"\" status=\"ok\">\n    <updatecheck codebase=\"{}\"",
                    escape(codebase)
                ));
                if let Some(sha256) = sha256 {
                    out.push_str(&format!(" hash_sha256=\"{}\"", escape(sha256)));
                }
                if let Some(size) = size {
                    out.push_str(&format!(" size=\"{size}\""));
                }
                out.push_str(&format!(
                    " status=\"ok\" version=\"{}\"/>\n  </app>\n",
                    escape(version)
                ));
            }
            Answer::NoUpdate => out.push_str(&format!(
                "  <app appid=\"{id}\" cohort=\"\" cohortname=\"\" status=\"ok\">\n    <updatecheck status=\"noupdate\"/>\n  </app>\n"
            )),
            Answer::Unknown => out.push_str(&format!(
                "  <app appid=\"{id}\" status=\"error-unknownApplication\"/>\n"
            )),
        }
    }
    out.push_str("</gupdate>\n");
    out
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RemoteApp {
    pub status: String,
    pub check_status: String,
    pub version: Option<String>,
    pub codebase: Option<String>,
    pub sha256: Option<String>,
    pub size: Option<u64>,
}

impl RemoteApp {
    pub fn offered(&self) -> Option<(&str, &str)> {
        let ok = |status: &str| status.is_empty() || status == "ok";
        if !ok(&self.status) || !ok(&self.check_status) {
            return None;
        }
        let version = self
            .version
            .as_deref()
            .filter(|version| valid_version(version))?;
        let codebase = self.codebase.as_deref().filter(|codebase| {
            url::Url::parse(codebase).is_ok_and(|url| matches!(url.scheme(), "http" | "https"))
        })?;
        Some((version, codebase))
    }
}

fn attribute(element: &BytesStart<'_>, wanted: &str) -> Option<String> {
    element
        .attributes()
        .flatten()
        .find(|found| {
            found
                .key
                .local_name()
                .into_inner()
                .eq_ignore_ascii_case(wanted)
        })
        .and_then(|found| {
            found
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .ok()
                .map(|value| value.trim().to_owned())
        })
}

fn checked_hash(hash: Option<String>) -> Option<String> {
    hash.map(|hash| hash.to_ascii_lowercase())
        .filter(|hash| hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()))
}

pub fn parse_answer(bytes: &[u8]) -> Result<BTreeMap<String, RemoteApp>, String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| "the update answer is not UTF-8".to_owned())?
        .trim_start_matches('\u{feff}');
    let mut reader = Reader::from_str(text);
    let mut depth = 0usize;
    let mut rooted = false;
    let mut current: Option<(String, RemoteApp)> = None;
    let mut apps = BTreeMap::new();
    let broken = |error: quick_xml::Error| format!("the update answer is not valid XML: {error}");
    loop {
        let event = reader.read_event().map_err(broken)?;
        let (element, empty) = match &event {
            Event::Start(start) => (Some(start.clone()), false),
            Event::Empty(empty) => (Some(empty.clone()), true),
            Event::End(_) => {
                depth = depth.saturating_sub(1);
                if depth == 1
                    && let Some((id, app)) = current.take()
                {
                    apps.insert(id, app);
                }
                continue;
            }
            Event::Eof => break,
            _ => continue,
        };
        let Some(element) = element else { continue };
        let name = element.local_name().into_inner().to_owned();
        if depth == 0 {
            rooted = name == "gupdate";
        }
        if depth == 1 && name == "app" {
            let id = attribute(&element, "appid")
                .unwrap_or_default()
                .to_ascii_lowercase();
            let app = RemoteApp {
                status: attribute(&element, "status").unwrap_or_default(),
                ..Default::default()
            };
            match empty {
                true if valid_id(&id) => {
                    apps.insert(id, app);
                }
                true => {}
                false => current = valid_id(&id).then_some((id, app)),
            }
        }
        if depth == 2
            && name == "updatecheck"
            && let Some((_, app)) = current.as_mut()
        {
            app.check_status = attribute(&element, "status").unwrap_or_default();
            app.version = attribute(&element, "version");
            app.codebase = attribute(&element, "codebase");
            app.sha256 = checked_hash(attribute(&element, "hash_sha256"));
            app.size = attribute(&element, "size").and_then(|size| size.parse().ok());
        }
        if !empty {
            depth += 1;
            if depth > MAX_DEPTH {
                return Err("the update answer nests too deeply".to_owned());
            }
        }
    }
    if !rooted {
        return Err("the update answer has no gupdate element".to_owned());
    }
    Ok(apps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const OTHER: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[test]
    fn requests_carry_one_entry_per_extension() {
        let query = format!(
            "response=updatecheck&os=linux&prodversion=131.0.6778.85&acceptformat=crx3&x=id%3D{ID}%26v%3D1.2%26installsource%3Dpolicy%26uc&x=id%3D{OTHER}%26v%3D%26uc&x=id%3D{ID}%26uc&x=id%3Dbad%26uc"
        );
        let request = parse_request(Some(&query)).unwrap();
        assert_eq!(request.mode, Mode::UpdateCheck);
        assert_eq!(request.prodversion.as_deref(), Some("131.0.6778.85"));
        assert_eq!(
            request.wanted,
            [
                Wanted {
                    id: ID.to_owned(),
                    version: Some("1.2".to_owned())
                },
                Wanted {
                    id: OTHER.to_owned(),
                    version: None
                },
            ]
        );
        let redirect = parse_request(Some(&format!(
            "response=redirect&prodversion=x&x=id%3D{ID}%26uc"
        )))
        .unwrap();
        assert_eq!(redirect.mode, Mode::Redirect);
        assert_eq!(redirect.prodversion, None);
        assert!(parse_request(Some("response=updatecheck")).is_err());
        assert!(parse_request(Some(&format!("response=ping&x=id%3D{ID}"))).is_err());
        assert!(is_update_request(Some("x=1")));
        assert!(!is_update_request(Some("q=1")));
        assert!(!is_update_request(None));
    }

    #[test]
    fn documents_answer_in_the_protocol_layout() {
        let now = Utc.with_ymd_and_hms(2026, 10, 24, 1, 2, 3).unwrap();
        let text = document(
            &[
                (
                    ID.to_owned(),
                    Answer::Update {
                        codebase: "https://r.test/repository/c/a&b.crx".to_owned(),
                        version: "1.2".to_owned(),
                        sha256: Some("ab".repeat(32)),
                        size: Some(42),
                    },
                ),
                (OTHER.to_owned(), Answer::NoUpdate),
                (
                    "cccccccccccccccccccccccccccccccc".to_owned(),
                    Answer::Unknown,
                ),
            ],
            now,
        );
        assert!(text.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<gupdate xmlns=\"http://www.google.com/update2/response\" protocol=\"2.0\""));
        assert!(text.contains("<daystart elapsed_seconds=\"3723\" elapsed_days=\"7236\"/>"));
        assert!(text.contains(&format!(
            "<updatecheck codebase=\"https://r.test/repository/c/a&amp;b.crx\" hash_sha256=\"{}\" size=\"42\" status=\"ok\" version=\"1.2\"/>",
            "ab".repeat(32)
        )));
        assert!(text.contains("<updatecheck status=\"noupdate\"/>"));
        assert!(text.contains(
            "appid=\"cccccccccccccccccccccccccccccccc\" status=\"error-unknownApplication\"/>"
        ));
        let parsed = parse_answer(text.as_bytes()).unwrap();
        assert_eq!(
            parsed[ID].offered(),
            Some(("1.2", "https://r.test/repository/c/a&b.crx"))
        );
        assert_eq!(parsed[ID].size, Some(42));
        assert_eq!(parsed[OTHER].offered(), None);
        assert_eq!(
            parsed["cccccccccccccccccccccccccccccccc"].status,
            "error-unknownApplication"
        );
    }

    #[test]
    fn remote_answers_are_read_with_or_without_a_prefix() {
        let answer = format!(
            "<?xml version='1.0' encoding='UTF-8'?><g:gupdate xmlns:g='{NAMESPACE}' protocol='2.0' server='prod'><g:daystart elapsed_days='1'/><g:app appid='{ID}' status='ok'><g:updatecheck codebase='https://cdn.test/x.crx' fp='1.x' hash_sha256='{}' protected='0' size='7' status='ok' version='3.0.1'/></g:app><g:app appid='{OTHER}' status='error-unknownApplication'></g:app></g:gupdate>",
            "AB".repeat(32)
        );
        let parsed = parse_answer(answer.as_bytes()).unwrap();
        assert_eq!(
            parsed[ID].offered(),
            Some(("3.0.1", "https://cdn.test/x.crx"))
        );
        assert_eq!(parsed[ID].sha256.as_deref(), Some("ab".repeat(32).as_str()));
        assert_eq!(parsed[OTHER].offered(), None);
        assert!(parse_answer(b"<html/>").is_err());
        assert!(parse_answer(b"<gupdate><app").is_err());
        let relative = format!(
            "<gupdate><app appid='{ID}'><updatecheck codebase='/x.crx' status='ok' version='1'/></app></gupdate>"
        );
        assert_eq!(
            parse_answer(relative.as_bytes()).unwrap()[ID].offered(),
            None
        );
    }
}
