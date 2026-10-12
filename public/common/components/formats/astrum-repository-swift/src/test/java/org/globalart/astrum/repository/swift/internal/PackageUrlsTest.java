package org.globalart.astrum.repository.swift.internal;

import java.util.List;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

class PackageUrlsTest
{
  @Test
  void normalizesTheSpellingsOfOneRepository() {
    for (String url : List.of("https://github.com/mona/LinkedList", "https://github.com/mona/LinkedList.git",
        "https://github.com/mona/LinkedList/", "HTTPS://GitHub.com/mona/linkedlist", "http://github.com/mona/LinkedList",
        "git@github.com:mona/LinkedList.git", "ssh://git@github.com/mona/LinkedList.git",
        "ssh://git@github.com:22/mona/LinkedList", "https://user@github.com:443/mona/LinkedList?tab=readme",
        "github.com/mona/LinkedList")) {
      assertThat(url, PackageUrls.normalize(url), is("github.com/mona/linkedlist"));
    }
  }

  @Test
  void keepsDifferentRepositoriesApart() {
    assertThat(PackageUrls.normalize("https://gitlab.example.com:8443/group/sub/Lib"),
        is("gitlab.example.com:8443/group/sub/lib"));
    assertThat(PackageUrls.normalize(""), is(nullValue()));
    assertThat(PackageUrls.normalize(null), is(nullValue()));
    assertThat(PackageUrls.normalize("https:///x"), is(nullValue()));
    assertThat(PackageUrls.normalizeAll(List.of("https://github.com/a/b", "git@github.com:a/b.git", " ")),
        contains("github.com/a/b"));
  }
}
