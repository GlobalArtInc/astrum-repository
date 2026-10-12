use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Problem {
    pub id: String,
    pub message: String,
}

impl Problem {
    pub fn new(field: &str, message: impl Into<String>) -> Self {
        Self {
            id: format!("PARAMETER {field}"),
            message: message.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Problems(Vec<Problem>);

impl Problems {
    pub fn add(&mut self, field: &str, message: impl Into<String>) {
        self.0.push(Problem::new(field, message));
    }

    pub fn require<T>(&mut self, field: &str, value: &Option<T>) {
        if value.is_none() {
            self.add(field, "must not be null");
        }
    }

    pub fn require_text(&mut self, field: &str, value: &Option<String>) {
        if value.as_deref().is_none_or(|text| text.trim().is_empty()) {
            self.add(field, "must not be empty");
        }
    }

    pub fn range(&mut self, field: &str, value: Option<i32>, min: i32, max: i32) {
        match value {
            Some(value) if value < min => {
                self.add(field, format!("must be greater than or equal to {min}"))
            }
            Some(value) if value > max => {
                self.add(field, format!("must be less than or equal to {max}"))
            }
            _ => {}
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn finish(self) -> Result<(), Vec<Problem>> {
        match self.0.is_empty() {
            true => Ok(()),
            false => Err(self.0),
        }
    }
}

pub fn blank_to_none(value: &mut Option<String>) {
    if value.as_deref().is_some_and(|text| text.trim().is_empty()) {
        *value = None;
    }
}

pub fn web_url(value: &str) -> bool {
    url::Url::parse(value.trim()).is_ok_and(|url| {
        matches!(url.scheme(), "http" | "https")
            && url.host_str().is_some_and(|host| !host.is_empty())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn problems_name_the_parameter() {
        let mut problems = Problems::default();
        problems.require::<bool>("online", &None);
        problems.require_text("name", &Some("  ".to_owned()));
        problems.range("retries", Some(11), 0, 10);
        problems.range("timeout", Some(0), 1, 3600);
        problems.range("timeout", Some(30), 1, 3600);
        let found = problems.finish().unwrap_err();
        let ids: Vec<&str> = found.iter().map(|problem| problem.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "PARAMETER online",
                "PARAMETER name",
                "PARAMETER retries",
                "PARAMETER timeout"
            ]
        );
        assert_eq!(found[2].message, "must be less than or equal to 10");
        assert!(Problems::default().finish().is_ok());
    }

    #[test]
    fn only_http_and_https_urls_are_web_urls() {
        for good in [
            "https://repo1.maven.org/maven2/",
            "http://10.0.0.1:8080",
            "HTTPS://Example.org",
        ] {
            assert!(web_url(good), "{good}");
        }
        for bad in [
            "ftp://example.org/",
            "not a url",
            "https://",
            "file:///etc",
            "",
        ] {
            assert!(!web_url(bad), "{bad}");
        }
    }

    #[test]
    fn blank_text_becomes_nothing() {
        let mut value = Some(" ".to_owned());
        blank_to_none(&mut value);
        assert_eq!(value, None);
        let mut value = Some("x".to_owned());
        blank_to_none(&mut value);
        assert_eq!(value.as_deref(), Some("x"));
    }
}
