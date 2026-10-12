package org.globalart.astrum.repository.swift.internal.proxy;

import java.net.URI;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.swift.AssetKind;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

class SwiftProxyFacetTest
{
  @Test
  void asksForTheRegistryMediaTypes() {
    assertThat(SwiftProxyFacet.accept(AssetKind.RELEASES), is("application/vnd.swift.registry.v1+json"));
    assertThat(SwiftProxyFacet.accept(AssetKind.RELEASE), is("application/vnd.swift.registry.v1+json"));
    assertThat(SwiftProxyFacet.accept(AssetKind.MANIFEST), is("application/vnd.swift.registry.v1+swift"));
    assertThat(SwiftProxyFacet.accept(AssetKind.ARCHIVE), is("application/vnd.swift.registry.v1+zip"));
  }

  @Test
  void readsManifestAlternatesOfTheRemote() {
    Map<String, String> alternates = SwiftProxyFacet.alternates(List.of(
        "<https://r.example.com/a/b/1.0.0/Package.swift?swift-version=4>; rel=\"alternate\"; "
            + "filename=\"Package@swift-4.swift\"; swift-tools-version=\"4.0\", "
            + "<https://r.example.com/a/b/1.0.0/Package.swift?swift-version=5.5>; rel=\"alternate\"; "
            + "filename=\"Package@swift-5.5.swift\"; swift-tools-version=\"bogus\"",
        "<https://r.example.com/x>; rel=\"alternate\"; filename=\"../../etc/passwd\"; swift-tools-version=\"5\""));
    assertThat(alternates, is(Map.of("Package@swift-4.swift", "4.0", "Package@swift-5.5.swift", "5.5")));
  }

  @Test
  void followsReleaseListPages() {
    assertThat(SwiftProxyFacet.nextPage(List.of("<https://r.example.com/a/b?page=2>; rel=\"next\", "
        + "<https://r.example.com/a/b?page=9>; rel=\"last\"")), is("https://r.example.com/a/b?page=2"));
    assertThat(SwiftProxyFacet.nextPage(List.of("<https://r.example.com/a/b/1.0.0>; rel=\"latest-version\"")),
        is(nullValue()));
    URI remote = URI.create("https://r.example.com/registry/");
    assertThat(SwiftProxyFacet.sameHost(URI.create("https://r.example.com/registry/a/b?page=2"), remote), is(true));
    assertThat(SwiftProxyFacet.sameHost(URI.create("https://evil.example.com/a/b?page=2"), remote), is(false));
  }
}
