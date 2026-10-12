package org.globalart.astrum.repository.swift.internal;

import java.util.List;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;

class LinksTest
{
  @Test
  void writesLinksAsTheClientReadsThem() {
    Links links = new Links()
        .add("https://r.example.com/mona/LinkedList/1.1.1", "latest-version")
        .add("git@github.com:mona/LinkedList.git", "alternate")
        .add("https://bad.example.com/a,b", "alternate")
        .addManifest("https://r.example.com/mona/LinkedList/1.1.1/Package.swift?swift-version=4.2",
            "Package@swift-4.2.swift", "4.2");
    String header = links.toString();
    assertThat(header, is("<https://r.example.com/mona/LinkedList/1.1.1>; rel=\"latest-version\", "
        + "<git@github.com:mona/LinkedList.git>; rel=\"alternate\", "
        + "<https://r.example.com/mona/LinkedList/1.1.1/Package.swift?swift-version=4.2>; rel=\"alternate\"; "
        + "filename=\"Package@swift-4.2.swift\"; swift-tools-version=\"4.2\""));
    for (String line : header.split(",")) {
      int fields = line.split(";").length;
      assertThat(line, fields == 2 || fields == 4, is(true));
    }
  }

  @Test
  void parsesLinkHeaders() {
    List<Links.Link> links = Links.parse(List.of(
        "<https://github.com/mona/LinkedList>; rel=\"canonical\", <ssh://git@github.com:mona/LinkedList.git>; rel=\"alternate\"",
        "<https://r.example.com/mona/HashMap?page=2>; rel=next",
        "<https://r.example.com/x/Package.swift?swift-version=5>; rel=\"alternate\"; filename=\"Package@swift-5.swift\"; swift-tools-version=\"5.0\""));
    assertThat(links, hasSize(4));
    assertThat(links.get(0).url(), is("https://github.com/mona/LinkedList"));
    assertThat(links.get(0).rel(), is("canonical"));
    assertThat(links.get(1).rel(), is("alternate"));
    assertThat(links.get(2).rel(), is("next"));
    assertThat(links.get(2).url(), is("https://r.example.com/mona/HashMap?page=2"));
    assertThat(links.get(3).param("filename"), is("Package@swift-5.swift"));
    assertThat(links.get(3).param("swift-tools-version"), is("5.0"));
    assertThat(Links.parse(null), hasSize(0));
  }
}
