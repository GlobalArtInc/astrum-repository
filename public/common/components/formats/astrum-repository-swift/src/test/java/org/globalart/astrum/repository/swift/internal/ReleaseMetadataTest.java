package org.globalart.astrum.repository.swift.internal;

import org.globalart.astrum.repository.swift.SwiftCoordinates;
import org.globalart.astrum.repository.swift.internal.ReleaseMetadata.Signing;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.junit.jupiter.api.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.matchesPattern;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;

class ReleaseMetadataTest
{
  static final String METADATA = "{\"description\":\"One thing links to another.\","
      + "\"repositoryURLs\":[\"https://github.com/mona/LinkedList\",\"git@github.com:mona/LinkedList.git\"],"
      + "\"licenseURL\":\"https://www.apache.org/licenses/LICENSE-2.0\","
      + "\"originalPublicationTime\":\"2023-02-16T04:00:00.123+02:00\","
      + "\"author\":{\"name\":\"Mona Lisa Octocat\",\"organization\":{\"name\":\"GitHub\"}},\"extra\":1}";

  @Test
  void acceptsTheSchema() {
    ObjectNode metadata = ReleaseMetadata.parse(METADATA.getBytes(UTF_8));
    assertThat(ReleaseMetadata.repositoryUrls(metadata),
        contains("https://github.com/mona/LinkedList", "git@github.com:mona/LinkedList.git"));
    assertThat(ReleaseMetadata.text(metadata, "description"), is("One thing links to another."));
    assertThat(ReleaseMetadata.parse(null).size(), is(0));
    assertThat(ReleaseMetadata.parse(new byte[0]).size(), is(0));
  }

  @Test
  void refusesInvalidMetadata() {
    for (String invalid : new String[]{"{", "[]", "\"x\"", "{\"description\":1}", "{\"repositoryURLs\":\"x\"}",
        "{\"repositoryURLs\":[1]}", "{\"author\":{}}", "{\"author\":{\"name\":\"a\",\"organization\":{}}}",
        "{\"originalPublicationTime\":\"yesterday\"}"}) {
      SwiftProblem problem = assertThrows(SwiftProblem.class, () -> ReleaseMetadata.parse(invalid.getBytes(UTF_8)),
          invalid);
      assertThat(invalid, problem.getStatus(), is(422));
    }
  }

  @Test
  void buildsReleaseInformation() {
    ObjectNode metadata = ReleaseMetadata.parse(METADATA.getBytes(UTF_8));
    ObjectNode release = ReleaseMetadata.release(new SwiftCoordinates("mona", "LinkedList", "1.1.1"),
        "a2ac54cf25fbc1ad0028f03f0aa4b96833b83bb05a14e510892bb27dea4dc812",
        new Signing("c2lnbmF0dXJl", "cms-1.0.0"), metadata, "2023-02-16T04:00:00Z");
    assertThat(release.get("id").asText(), is("mona.LinkedList"));
    assertThat(release.get("version").asText(), is("1.1.1"));
    JsonNode resource = release.get("resources").get(0);
    assertThat(resource.get("name").asText(), is("source-archive"));
    assertThat(resource.get("type").asText(), is("application/zip"));
    assertThat(resource.get("checksum").asText(), is("a2ac54cf25fbc1ad0028f03f0aa4b96833b83bb05a14e510892bb27dea4dc812"));
    assertThat(resource.get("signing").get("signatureFormat").asText(), is("cms-1.0.0"));
    assertThat(release.get("metadata").get("originalPublicationTime").asText(), is("2023-02-16T02:00:00Z"));
    assertThat(release.get("publishedAt").asText(), is("2023-02-16T04:00:00Z"));
    assertThat(ReleaseMetadata.signing(release), is(new Signing("c2lnbmF0dXJl", "cms-1.0.0")));
    assertThat(ReleaseMetadata.checksum(release), is(resource.get("checksum").asText()));
    assertThat(metadata.get("originalPublicationTime").asText(), is("2023-02-16T04:00:00.123+02:00"));
  }

  @Test
  void writesTimesWithoutFractions() {
    assertThat(ReleaseMetadata.now(), matchesPattern("\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}Z"));
    assertThat(ReleaseMetadata.parseTime("2023-02-16T04:00:00.000Z"), is("2023-02-16T04:00:00Z"));
    assertThat(ReleaseMetadata.parseTime("16.02.2023"), is(nullValue()));
  }
}
