package org.globalart.astrum.repository.swift.internal;

import java.io.IOException;
import java.io.InputStream;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.swift.SwiftCoordinates;
import org.globalart.astrum.repository.swift.internal.ReleaseMetadata.Signing;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import com.fasterxml.jackson.databind.JsonNode;
import org.junit.jupiter.api.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

class ReleaseResponsesTest
{
  private static final String BASE = "https://repo.example.com/repository/swift-hosted";

  private static final SwiftCoordinates PACKAGE = new SwiftCoordinates("mona", "LinkedList", null);

  @Test
  void listsReleasesByPrecedenceWithLinks() throws IOException {
    Map<String, JsonNode> versions = new LinkedHashMap<>();
    versions.put("1.0.0", null);
    versions.put("1.1.1", null);
    versions.put("2.0.0", SwiftResponses.MAPPER.readTree("{\"status\":410,\"detail\":\"removed\"}"));
    versions.put("1.1.0", null);
    Response response = ReleaseResponses.releases(BASE, PACKAGE, versions,
        List.of(Map.of("url", "https://github.com/mona/LinkedList", "rel", "canonical")));
    assertThat(response.getStatus().getCode(), is(200));
    assertThat(response.getHeaders().get("Content-Version"), is("1"));
    assertThat(response.getPayload().getContentType(), is("application/json"));
    JsonNode body = read(response);
    List<String> order = new java.util.ArrayList<>();
    body.get("releases").fieldNames().forEachRemaining(order::add);
    assertThat(order, contains("2.0.0", "1.1.1", "1.1.0", "1.0.0"));
    assertThat(body.get("releases").get("1.1.1").get("url").asText(), is(BASE + "/mona/LinkedList/1.1.1"));
    assertThat(body.get("releases").get("2.0.0").get("problem").get("status").asInt(), is(410));
    assertThat(response.getHeaders().get("Link"), is("<" + BASE + "/mona/LinkedList/1.1.1>; rel=\"latest-version\", "
        + "<https://github.com/mona/LinkedList>; rel=\"canonical\""));
  }

  @Test
  void linksNeighbouringReleases() {
    SwiftCoordinates release = PACKAGE.withVersion("1.1.0");
    List<String> versions = List.of("1.0.0", "1.1.1", "1.1.0");
    assertThat(ReleaseResponses.releaseLinks(BASE, release, versions).toString(),
        is("<" + BASE + "/mona/LinkedList/1.1.1>; rel=\"latest-version\", "
            + "<" + BASE + "/mona/LinkedList/1.1.1>; rel=\"successor-version\", "
            + "<" + BASE + "/mona/LinkedList/1.0.0>; rel=\"predecessor-version\""));
    assertThat(ReleaseResponses.releaseLinks(BASE, PACKAGE.withVersion("1.1.1"), versions).toString(),
        is("<" + BASE + "/mona/LinkedList/1.1.1>; rel=\"latest-version\", "
            + "<" + BASE + "/mona/LinkedList/1.1.0>; rel=\"predecessor-version\""));
  }

  @Test
  void answersManifestsWithAlternates() {
    Map<String, String> alternates = new LinkedHashMap<>();
    alternates.put("Package@swift-5.swift", "5.0");
    alternates.put("Package@swift-4.2.swift", "");
    Response response = ReleaseResponses.manifest(BASE, PACKAGE.withVersion("1.1.1"), "Package.swift",
        new Content(new BytesPayload("// swift-tools-version:5.9".getBytes(UTF_8), "text/x-swift")), alternates);
    assertThat(response.getHeaders().get("Content-Disposition"), is("attachment; filename=\"Package.swift\""));
    assertThat(response.getHeaders().get("Cache-Control"), is("public, immutable"));
    assertThat(response.getHeaders().get("Link"),
        is("<" + BASE + "/mona/LinkedList/1.1.1/Package.swift?swift-version=4.2>; rel=\"alternate\"; "
            + "filename=\"Package@swift-4.2.swift\"; swift-tools-version=\"4.2\", "
            + "<" + BASE + "/mona/LinkedList/1.1.1/Package.swift?swift-version=5>; rel=\"alternate\"; "
            + "filename=\"Package@swift-5.swift\"; swift-tools-version=\"5.0\""));
    Response variant = ReleaseResponses.manifest(BASE, PACKAGE.withVersion("1.1.1"), "Package@swift-5.swift",
        new Content(new BytesPayload(new byte[0], "text/x-swift")), alternates);
    assertThat(variant.getHeaders().get("Link"), is(nullValue()));
  }

  @Test
  void answersArchivesWithDigestAndSignature() {
    Response response = ReleaseResponses.archive(PACKAGE.withVersion("1.1.1"),
        new Content(new BytesPayload(new byte[0], "application/zip")),
        "a2ac54cf25fbc1ad0028f03f0aa4b96833b83bb05a14e510892bb27dea4dc812", new Signing("c2ln", "cms-1.0.0"));
    assertThat(response.getHeaders().get("Content-Disposition"), is("attachment; filename=\"LinkedList-1.1.1.zip\""));
    assertThat(response.getHeaders().get("Digest"), is("sha-256=oqxUzyX7wa0AKPA/CqS5aDO4O7BaFOUQiSuyfepNyBI="));
    assertThat(response.getHeaders().get("X-Swift-Package-Signature-Format"), is("cms-1.0.0"));
    assertThat(response.getHeaders().get("X-Swift-Package-Signature"), is("c2ln"));
    assertThat(ReleaseResponses.digest("not-hex"), is(nullValue()));
  }

  @Test
  void writesProblemDetails() throws IOException {
    Response problem = SwiftResponses.problem(404, "release not found");
    assertThat(problem.getStatus().getCode(), is(404));
    assertThat(problem.getHeaders().get("Content-Version"), is("1"));
    assertThat(problem.getHeaders().get("Content-Language"), is("en"));
    assertThat(problem.getPayload().getContentType(), is("application/problem+json"));
    JsonNode body = read(problem);
    assertThat(body.get("detail").asText(), is("release not found"));
    assertThat(body.get("status").asInt(), is(404));
  }

  static JsonNode read(final Response response) throws IOException {
    try (InputStream in = response.getPayload().openInputStream()) {
      return SwiftResponses.MAPPER.readTree(in);
    }
  }
}
