package org.globalart.astrum.repository.swift.internal.hosted;

import java.io.IOException;
import java.io.InputStream;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.swift.SwiftCoordinates;
import org.globalart.astrum.repository.swift.internal.SwiftApiHandler;
import org.globalart.astrum.repository.swift.internal.SwiftCommonHandlers;
import org.globalart.astrum.repository.swift.internal.SwiftContentFacet;
import org.globalart.astrum.repository.swift.internal.SwiftPaths;
import org.globalart.astrum.repository.swift.internal.SwiftResponses;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import com.fasterxml.jackson.databind.JsonNode;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.argThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class SwiftHostedHandlersTest
{
  private static final String BASE = "https://repo.example.com/repository/swift-hosted";

  private final Repository repository = mock(Repository.class);

  private final SwiftContentFacet content = mock(SwiftContentFacet.class);

  private Router router;

  @BeforeEach
  void setUp() {
    when(repository.getUrl()).thenReturn(BASE);
    when(repository.getName()).thenReturn("swift-hosted");
    when(repository.facet(SwiftContentFacet.class)).thenReturn(content);
    SwiftHostedHandlers hosted = new SwiftHostedHandlers();
    SwiftApiHandler api = new SwiftApiHandler();
    router = new Router.Builder()
        .route(route(SwiftPaths.identifiers(), api, hosted.identifiers))
        .route(route(SwiftPaths.archive(), api, hosted.archive))
        .route(route(SwiftPaths.manifest(), api, hosted.manifest))
        .route(route(SwiftPaths.releases(), api, hosted.releases))
        .route(route(SwiftPaths.release(), api, hosted.release))
        .defaultHandlers(api, new SwiftCommonHandlers().notFound)
        .create();

    FluentComponent v100 = component("1.0.0", Map.of());
    FluentComponent v111 = component("1.1.1", Map.of("description", "One thing links to another.",
        "repositoryURLs", List.of("https://github.com/mona/LinkedList")));
    when(content.releases(argThat(c -> c != null && c.samePackage("mona", "LinkedList")))).thenReturn(List.of(v111, v100));
    when(content.release(any())).thenReturn(Optional.empty());
    when(content.release(argThat(c -> c != null && c.samePackage("mona", "LinkedList") && "1.1.1".equals(c.getVersion()))))
        .thenReturn(Optional.of(v111));
    FluentAsset manifest = mock(FluentAsset.class);
    Map<String, Object> alternates = new HashMap<>(Map.of("Package@swift-4.2.swift", "4.2"));
    when(manifest.attributes()).thenReturn(new NestedAttributesMap("attributes",
        new HashMap<>(Map.of("swift", new HashMap<>(Map.of("alternates", alternates))))));
    when(manifest.download()).thenReturn(new Content(new BytesPayload("// swift-tools-version:5.9\n".getBytes(UTF_8),
        "text/x-swift")));
    when(content.getAsset(any())).thenReturn(Optional.empty());
    when(content.getAsset("/mona/LinkedList/1.1.1/Package.swift")).thenReturn(Optional.of(manifest));
    when(content.identifiers("https://github.com/mona/LinkedList.git")).thenReturn(List.of("mona.LinkedList"));
  }

  private static Route route(final Matcher matcher, final Handler... handlers) {
    Route.Builder builder = new Route.Builder().matcher(matcher);
    for (Handler handler : handlers) {
      builder.handler(handler);
    }
    return builder.create();
  }

  private static FluentComponent component(final String version, final Map<String, Object> extra) {
    FluentComponent component = mock(FluentComponent.class);
    when(component.namespace()).thenReturn("mona");
    when(component.name()).thenReturn("LinkedList");
    when(component.version()).thenReturn(version);
    Map<String, Object> swift = new LinkedHashMap<>(extra);
    swift.put("checksum", "a2ac54cf25fbc1ad0028f03f0aa4b96833b83bb05a14e510892bb27dea4dc812");
    swift.put("publishedAt", "2023-02-16T04:00:00Z");
    swift.put("metadata", new LinkedHashMap<>(extra));
    when(component.attributes()).thenReturn(new NestedAttributesMap("attributes", new HashMap<>(Map.of("swift", swift))));
    return component;
  }

  private Response get(final String path, final String accept, final String... parameters) throws Exception {
    Request.Builder request = new Request.Builder().action("GET").path(path);
    if (accept != null) {
      request.header("Accept", accept);
    }
    for (int i = 0; i < parameters.length; i += 2) {
      request.parameter(parameters[i], parameters[i + 1]);
    }
    return router.dispatch(repository, request.build(), null);
  }

  private static JsonNode json(final Response response) throws IOException {
    try (InputStream in = response.getPayload().openInputStream()) {
      return SwiftResponses.MAPPER.readTree(in);
    }
  }

  @Test
  void listsReleases() throws Exception {
    Response response = get("/MONA/linkedlist", "application/vnd.swift.registry.v1+json");
    assertThat(response.getStatus().getCode(), is(200));
    assertThat(response.getHeaders().get("Content-Version"), is("1"));
    assertThat(json(response).toString(), is("{\"releases\":{"
        + "\"1.1.1\":{\"url\":\"" + BASE + "/mona/LinkedList/1.1.1\"},"
        + "\"1.0.0\":{\"url\":\"" + BASE + "/mona/LinkedList/1.0.0\"}}}"));
    assertThat(response.getHeaders().get("Link"), is("<" + BASE + "/mona/LinkedList/1.1.1>; rel=\"latest-version\", "
        + "<https://github.com/mona/LinkedList>; rel=\"canonical\""));
  }

  @Test
  void describesReleases() throws Exception {
    Response response = get("/mona/LinkedList/1.1.1.json", null);
    assertThat(response.getStatus().getCode(), is(200));
    JsonNode body = json(response);
    assertThat(body.get("id").asText(), is("mona.LinkedList"));
    assertThat(body.get("version").asText(), is("1.1.1"));
    assertThat(body.get("resources").get(0).get("checksum").asText(),
        is("a2ac54cf25fbc1ad0028f03f0aa4b96833b83bb05a14e510892bb27dea4dc812"));
    assertThat(body.get("metadata").get("description").asText(), is("One thing links to another."));
    assertThat(body.get("publishedAt").asText(), is("2023-02-16T04:00:00Z"));
    assertThat(response.getHeaders().get("Link"), is("<" + BASE + "/mona/LinkedList/1.1.1>; rel=\"latest-version\", "
        + "<" + BASE + "/mona/LinkedList/1.0.0>; rel=\"predecessor-version\""));
  }

  @Test
  void answersManifestsAndRedirectsUnknownSwiftVersions() throws Exception {
    Response manifest = get("/mona/LinkedList/1.1.1/Package.swift", "application/vnd.swift.registry.v1+swift");
    assertThat(manifest.getStatus().getCode(), is(200));
    assertThat(manifest.getPayload().getContentType(), is("text/x-swift"));
    assertThat(manifest.getHeaders().get("Link"), is("<" + BASE + "/mona/LinkedList/1.1.1/Package.swift"
        + "?swift-version=4.2>; rel=\"alternate\"; filename=\"Package@swift-4.2.swift\"; swift-tools-version=\"4.2\""));
    Response redirect = get("/mona/LinkedList/1.1.1/Package.swift", null, "swift-version", "6.0");
    assertThat(redirect.getStatus().getCode(), is(303));
    assertThat(redirect.getHeaders().get("Location"), is(BASE + "/mona/LinkedList/1.1.1/Package.swift"));
    Response missingVariant = get("/mona/LinkedList/1.1.1/Package.swift", null, "swift-version", "4.2.0");
    assertThat(missingVariant.getStatus().getCode(), is(404));
  }

  @Test
  void answersProblemDetails() throws Exception {
    Response missing = get("/mona/LinkedList/9.9.9", "application/vnd.swift.registry.v1+json");
    assertThat(missing.getStatus().getCode(), is(404));
    assertThat(missing.getPayload().getContentType(), is("application/problem+json"));
    assertThat(json(missing).get("detail").asText(), is("release 9.9.9 of mona.LinkedList not found"));
    assertThat(get("/mona/LinkedList/9.9.9.zip", null).getStatus().getCode(), is(404));
    assertThat(get("/mona/LinkedList", "application/vnd.swift.registry.v2+json").getStatus().getCode(), is(415));
    assertThat(get("/mona/LinkedList", "application/vnd.swift.registry.vlatest+json").getStatus().getCode(), is(400));
    Response unknown = get("/a/b/c/d/e", null);
    assertThat(unknown.getStatus().getCode(), is(404));
    assertThat(unknown.getHeaders().get("Content-Version"), is("1"));
  }

  @Test
  void looksUpIdentifiers() throws Exception {
    Response response = get("/identifiers", "application/vnd.swift.registry.v1+json",
        "url", "https://github.com/mona/LinkedList.git");
    assertThat(response.getStatus().getCode(), is(200));
    assertThat(json(response).toString(), is("{\"identifiers\":[\"mona.LinkedList\"]}"));
    assertThat(get("/identifiers", null).getStatus().getCode(), is(400));
    assertThat(get("/identifiers", null, "url", "https://github.com/nobody/x").getStatus().getCode(), is(404));
  }

  @Test
  void matchesOnlyValidCoordinates() throws Exception {
    assertThat(get("/-mona/LinkedList", null).getStatus().getCode(), is(404));
    assertThat(get("/mona/LinkedList/latest", null).getStatus().getCode(), is(404));
    assertThat(new SwiftCoordinates("mona", "LinkedList", "1.1.1").getArchivePath(), is("/mona/LinkedList/1.1.1.zip"));
  }
}
