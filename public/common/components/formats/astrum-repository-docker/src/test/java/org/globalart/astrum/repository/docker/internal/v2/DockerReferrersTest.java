package org.globalart.astrum.repository.docker.internal.v2;

import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.docker.internal.security.DockerAuth;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;

import com.fasterxml.jackson.databind.JsonNode;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.Matchers.startsWith;
import static org.mockito.Mockito.mock;

public class DockerReferrersTest
{
  private static final String SUBJECT = "sha256:" + "0".repeat(64);

  private static String digest(final int i) {
    return "sha256:" + Integer.toHexString(i).repeat(64).substring(0, 64);
  }

  private static Map<String, Object> descriptor(final String digest, final String artifactType) {
    Map<String, Object> descriptor = new LinkedHashMap<>();
    descriptor.put("mediaType", DockerManifest.OCI_MANIFEST);
    descriptor.put("digest", digest);
    descriptor.put("size", 100);
    descriptor.put("artifactType", artifactType);
    return descriptor;
  }

  private static Context context(final String... parameters) {
    Request.Builder builder = new Request.Builder()
        .action("GET")
        .path("/v2/app/referrers/" + SUBJECT)
        .attribute(DockerAuth.PREFIX_ATTRIBUTE, "/v2/hosted/");
    Request request = builder.build();
    for (int i = 0; i < parameters.length; i += 2) {
      request.getParameters().set(parameters[i], parameters[i + 1]);
    }
    return new Context(mock(Repository.class), request);
  }

  private static JsonNode body(final Response response) throws Exception {
    try (InputStream in = response.getPayload().openInputStream()) {
      return DockerResponses.MAPPER.readTree(in);
    }
  }

  @Test
  public void mergeKeepsTheFirstDescriptorOfEachDigest() {
    Map<String, Object> first = descriptor(digest(1), "application/a");
    Map<String, Object> again = descriptor(digest(1), "application/b");
    Map<String, Object> second = descriptor(digest(2), "application/b");
    List<Map<String, Object>> merged = DockerReferrers.merge(List.of(List.of(first), List.of(again, second)));
    assertThat(merged, contains(first, second));
  }

  @Test
  public void filterByArtifactType() {
    List<Map<String, Object>> descriptors =
        List.of(descriptor(digest(1), "application/spdx+json"), descriptor(digest(2), "application/other"));
    assertThat(DockerReferrers.filter(descriptors, "application/spdx+json"), hasSize(1));
    assertThat(DockerReferrers.filter(descriptors, null), hasSize(2));
  }

  @Test
  public void responseIsAnIndexAndReportsTheFilter() throws Exception {
    List<Map<String, Object>> descriptors =
        List.of(descriptor(digest(2), "application/spdx+json"), descriptor(digest(1), "application/other"));
    Response unfiltered = DockerReferrers.response(context(), "app", SUBJECT, descriptors, 4096);
    assertThat(unfiltered.getStatus().getCode(), is(200));
    assertThat(unfiltered.getPayload().getContentType(), is(DockerManifest.OCI_INDEX));
    assertThat(unfiltered.getHeaders().get(DockerReferrers.FILTERS_APPLIED_HEADER), nullValue());
    JsonNode index = body(unfiltered);
    assertThat(index.path("schemaVersion").asInt(), is(2));
    assertThat(index.path("mediaType").asText(), is(DockerManifest.OCI_INDEX));
    assertThat(index.path("manifests").get(0).path("digest").asText(), is(digest(1)));

    Response filtered = DockerReferrers.response(context("artifactType", "application/spdx+json"), "app", SUBJECT,
        descriptors, 4096);
    assertThat(filtered.getHeaders().get(DockerReferrers.FILTERS_APPLIED_HEADER), is("artifactType"));
    assertThat(body(filtered).path("manifests").size(), is(1));
  }

  @Test
  public void emptyListIsStillAnIndex() throws Exception {
    Response response = DockerReferrers.response(context(), "app", SUBJECT, List.of(), 4096);
    assertThat(response.getStatus().getCode(), is(200));
    assertThat(body(response).path("manifests").isArray(), is(true));
    assertThat(body(response).path("manifests").size(), is(0));
  }

  @Test
  public void largeAnswersArePaged() throws Exception {
    List<Map<String, Object>> descriptors = new ArrayList<>();
    for (int i = 1; i <= 9; i++) {
      descriptors.add(descriptor(digest(i), "application/other"));
    }
    Response first = DockerReferrers.response(context(), "app", SUBJECT, descriptors, 600);
    int firstPage = body(first).path("manifests").size();
    assertThat(firstPage < 9, is(true));
    String link = first.getHeaders().get("Link");
    assertThat(link, startsWith("</v2/hosted/app/referrers/" + SUBJECT + "?last="));
    String last = body(first).path("manifests").get(firstPage - 1).path("digest").asText();

    Response second = DockerReferrers.response(context("last", last), "app", SUBJECT, descriptors, 4096);
    assertThat(body(second).path("manifests").size(), is(9 - firstPage));
    assertThat(second.getHeaders().get("Link"), nullValue());
  }

  @Test
  public void parsesTheDescriptorsOfAnIndex() throws Exception {
    String json = "{\"schemaVersion\":2,\"mediaType\":\"" + DockerManifest.OCI_INDEX + "\",\"manifests\":["
        + "{\"mediaType\":\"" + DockerManifest.OCI_MANIFEST + "\",\"digest\":\"" + digest(3) + "\",\"size\":5,"
        + "\"artifactType\":\"application/vnd.example\",\"annotations\":{\"a\":\"b\"}},{\"digest\":\"nope\"}]}";
    List<Map<String, Object>> descriptors =
        DockerReferrers.parseIndex(new ByteArrayInputStream(json.getBytes(StandardCharsets.UTF_8)));
    assertThat(descriptors, hasSize(1));
    assertThat(descriptors.get(0).get("artifactType"), is("application/vnd.example"));
    assertThat(descriptors.get(0).get("annotations"), is(Map.of("a", "b")));
  }
}
