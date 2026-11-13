use std::collections::HashMap;
use std::sync::OnceLock;

const ALIASES: &[(&str, &str)] = &[
    ("Apache 2.0", "Apache-2.0"),
    ("Apache License 2.0", "Apache-2.0"),
    ("Apache License, Version 2.0", "Apache-2.0"),
    ("The Apache Software License, Version 2.0", "Apache-2.0"),
    ("ASL 2.0", "Apache-2.0"),
    ("https://www.apache.org/licenses/LICENSE-2.0", "Apache-2.0"),
    ("http://www.apache.org/licenses/LICENSE-2.0", "Apache-2.0"),
    ("MIT License", "MIT"),
    ("The MIT License", "MIT"),
    ("The MIT License (MIT)", "MIT"),
    ("https://opensource.org/licenses/MIT", "MIT"),
    ("https://opensource.org/license/mit", "MIT"),
    ("ISC License", "ISC"),
    ("https://opensource.org/licenses/ISC", "ISC"),
    ("BSD 3-Clause", "BSD-3-Clause"),
    ("BSD 3-Clause License", "BSD-3-Clause"),
    ("BSD-3", "BSD-3-Clause"),
    ("The 3-Clause BSD License", "BSD-3-Clause"),
    ("New BSD License", "BSD-3-Clause"),
    ("Modified BSD License", "BSD-3-Clause"),
    (
        "https://opensource.org/licenses/BSD-3-Clause",
        "BSD-3-Clause",
    ),
    ("BSD 2-Clause", "BSD-2-Clause"),
    ("BSD 2-Clause License", "BSD-2-Clause"),
    ("BSD-2", "BSD-2-Clause"),
    ("Simplified BSD License", "BSD-2-Clause"),
    ("The 2-Clause BSD License", "BSD-2-Clause"),
    ("FreeBSD", "BSD-2-Clause"),
    ("FreeBSD License", "BSD-2-Clause"),
    (
        "https://opensource.org/licenses/BSD-2-Clause",
        "BSD-2-Clause",
    ),
    ("GPLv2", "GPL-2.0-only"),
    ("GPL v2", "GPL-2.0-only"),
    ("GPL-2", "GPL-2.0-only"),
    ("GNU GPL v2", "GPL-2.0-only"),
    ("GNU General Public License v2.0", "GPL-2.0-only"),
    ("GNU General Public License, version 2", "GPL-2.0-only"),
    (
        "https://www.gnu.org/licenses/old-licenses/gpl-2.0.html",
        "GPL-2.0-only",
    ),
    ("GPLv2+", "GPL-2.0-or-later"),
    ("GPLv2 or later", "GPL-2.0-or-later"),
    ("GPL v2 or later", "GPL-2.0-or-later"),
    (
        "GNU General Public License v2.0 or later",
        "GPL-2.0-or-later",
    ),
    ("GPLv3", "GPL-3.0-only"),
    ("GPL v3", "GPL-3.0-only"),
    ("GPL-3", "GPL-3.0-only"),
    ("GNU General Public License v3.0", "GPL-3.0-only"),
    ("GNU General Public License, version 3", "GPL-3.0-only"),
    ("https://www.gnu.org/licenses/gpl-3.0.html", "GPL-3.0-only"),
    ("GPLv3+", "GPL-3.0-or-later"),
    ("GPLv3 or later", "GPL-3.0-or-later"),
    ("GPL v3 or later", "GPL-3.0-or-later"),
    (
        "GNU General Public License v3.0 or later",
        "GPL-3.0-or-later",
    ),
    ("LGPLv2.1", "LGPL-2.1-only"),
    ("LGPL 2.1", "LGPL-2.1-only"),
    ("GNU Lesser General Public License v2.1", "LGPL-2.1-only"),
    (
        "https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html",
        "LGPL-2.1-only",
    ),
    ("LGPLv2.1+", "LGPL-2.1-or-later"),
    ("LGPL v2.1 or later", "LGPL-2.1-or-later"),
    (
        "GNU Lesser General Public License v2.1 or later",
        "LGPL-2.1-or-later",
    ),
    ("LGPLv3", "LGPL-3.0-only"),
    ("LGPL v3", "LGPL-3.0-only"),
    (
        "https://www.gnu.org/licenses/lgpl-3.0.html",
        "LGPL-3.0-only",
    ),
    ("LGPLv3+", "LGPL-3.0-or-later"),
    ("LGPL v3 or later", "LGPL-3.0-or-later"),
    ("AGPLv3", "AGPL-3.0-only"),
    ("AGPL v3", "AGPL-3.0-only"),
    ("GNU Affero General Public License v3.0", "AGPL-3.0-only"),
    (
        "https://www.gnu.org/licenses/agpl-3.0.html",
        "AGPL-3.0-only",
    ),
    ("AGPLv3+", "AGPL-3.0-or-later"),
    (
        "GNU Affero General Public License v3.0 or later",
        "AGPL-3.0-or-later",
    ),
    ("MPL 2.0", "MPL-2.0"),
    ("Mozilla Public License 2.0", "MPL-2.0"),
    ("Mozilla Public License, v. 2.0", "MPL-2.0"),
    ("https://www.mozilla.org/en-US/MPL/2.0/", "MPL-2.0"),
    ("https://mozilla.org/MPL/2.0/", "MPL-2.0"),
    ("Eclipse Public License 1.0", "EPL-1.0"),
    ("EPL 1.0", "EPL-1.0"),
    ("https://www.eclipse.org/legal/epl-v10.html", "EPL-1.0"),
    ("Eclipse Public License 2.0", "EPL-2.0"),
    ("Eclipse Public License v2.0", "EPL-2.0"),
    ("EPL 2.0", "EPL-2.0"),
    ("https://www.eclipse.org/legal/epl-2.0/", "EPL-2.0"),
    ("CDDL 1.0", "CDDL-1.0"),
    (
        "Common Development and Distribution License 1.0",
        "CDDL-1.0",
    ),
    ("CDDL 1.1", "CDDL-1.1"),
    (
        "Common Development and Distribution License 1.1",
        "CDDL-1.1",
    ),
    ("The Unlicense", "Unlicense"),
    ("https://unlicense.org/", "Unlicense"),
    ("CC0", "CC0-1.0"),
    ("CC0 1.0", "CC0-1.0"),
    ("CC0 1.0 Universal", "CC0-1.0"),
    (
        "https://creativecommons.org/publicdomain/zero/1.0/",
        "CC0-1.0",
    ),
    ("Python Software Foundation License", "PSF-2.0"),
    ("Python License 2.0", "Python-2.0"),
    ("zlib License", "Zlib"),
    ("https://opensource.org/licenses/Zlib", "Zlib"),
    ("Boost Software License 1.0", "BSL-1.0"),
    ("https://www.boost.org/LICENSE_1_0.txt", "BSL-1.0"),
    ("Artistic License 2.0", "Artistic-2.0"),
    ("SIL Open Font License 1.1", "OFL-1.1"),
    ("Zero-Clause BSD", "0BSD"),
];

static TABLE: OnceLock<HashMap<String, &'static str>> = OnceLock::new();

pub fn normalise_key(text: &str) -> String {
    text.replace('+', " plus ")
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn lookup(text: &str) -> Option<&'static str> {
    table().get(&normalise_key(text)).copied()
}

fn table() -> &'static HashMap<String, &'static str> {
    TABLE.get_or_init(|| {
        ALIASES
            .iter()
            .map(|(spelling, id)| (normalise_key(spelling), *id))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::license::{Expression, State, normalise, spdx};

    #[test]
    fn aliases_and_urls_normalise_to_their_ids_in_mapped_state() {
        let cases = [
            ("Apache 2.0", "Apache-2.0"),
            ("Apache License, Version 2.0", "Apache-2.0"),
            ("The Apache Software License, Version 2.0", "Apache-2.0"),
            ("https://www.apache.org/licenses/LICENSE-2.0", "Apache-2.0"),
            ("MIT License", "MIT"),
            ("https://opensource.org/licenses/MIT", "MIT"),
            ("New BSD License", "BSD-3-Clause"),
            ("Simplified BSD License", "BSD-2-Clause"),
            ("GPLv2", "GPL-2.0-only"),
            ("GNU General Public License, version 2", "GPL-2.0-only"),
            ("GPLv3", "GPL-3.0-only"),
            ("GPL-3", "GPL-3.0-only"),
            ("GPLv2+", "GPL-2.0-or-later"),
            ("LGPLv2.1", "LGPL-2.1-only"),
            ("LGPLv3", "LGPL-3.0-only"),
            ("AGPLv3", "AGPL-3.0-only"),
            ("MPL 2.0", "MPL-2.0"),
            ("Mozilla Public License 2.0", "MPL-2.0"),
            ("Eclipse Public License 1.0", "EPL-1.0"),
            ("EPL 2.0", "EPL-2.0"),
            ("CDDL 1.1", "CDDL-1.1"),
            ("The Unlicense", "Unlicense"),
            ("CC0", "CC0-1.0"),
            ("Python Software Foundation License", "PSF-2.0"),
            ("ISC License", "ISC"),
        ];
        for (spelling, id) in cases {
            let normalised = normalise(spelling);
            assert_eq!(normalised.state, State::Mapped, "{spelling}");
            assert_eq!(
                normalised.expression,
                Some(Expression::Id(id.to_owned())),
                "{spelling}"
            );
        }
    }

    #[test]
    fn every_alias_is_mapped_to_a_listed_id_and_no_key_is_listed_twice() {
        assert_eq!(table().len(), ALIASES.len());
        for (spelling, id) in ALIASES {
            assert!(
                spdx::lookup(id).is_some_and(|entry| !entry.deprecated),
                "{id}"
            );
            assert_eq!(normalise(spelling).state, State::Mapped, "{spelling}");
        }
    }

    #[test]
    fn ambiguous_names_without_a_version_stay_unknown() {
        for spelling in [
            "BSD",
            "BSD License",
            "GPL",
            "LGPL",
            "AGPL",
            "GNU General Public License",
            "GNU Lesser General Public License",
            "CDDL",
            "Public Domain",
            "Dual License",
            "Proprietary",
            "Commercial",
            "Other",
        ] {
            assert_eq!(normalise(spelling).state, State::Unknown, "{spelling}");
        }
    }
}
