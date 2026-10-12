package org.globalart.astrum.repository.swift.internal.proxy;

import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.swift.SwiftCoordinates;
import org.globalart.astrum.repository.swift.SwiftVersion;
import org.globalart.astrum.repository.swift.internal.Links;
import org.globalart.astrum.repository.swift.internal.Manifests;
import org.globalart.astrum.repository.swift.internal.ReleaseResponses;
import org.globalart.astrum.repository.swift.internal.SwiftContentFacet;
import org.globalart.astrum.repository.swift.internal.SwiftHandlers;
import org.globalart.astrum.repository.swift.internal.SwiftPaths;
import org.globalart.astrum.repository.swift.internal.SwiftProblem;
import org.globalart.astrum.repository.swift.internal.SwiftResponses;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Parameters;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.ViewFacet;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.swift.internal.SwiftContentFacet.CHECKSUM;
import static org.globalart.astrum.repository.swift.internal.SwiftContentFacet.SIGNING;
import static org.globalart.astrum.repository.swift.internal.SwiftHandlers.content;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.LINK;

@Component
public class SwiftProxyHandlers
{
  private static final Logger log = LoggerFactory.getLogger(SwiftProxyHandlers.class);

  public final Handler releases = context -> {
    Response response = context.proceed();
    byte[] bytes = body(response);
    JsonNode body = SwiftResponses.readQuietly(bytes);
    if (body == null || !(body.get(ReleaseResponses.RELEASES) instanceof ObjectNode releases)) {
      return response;
    }
    SwiftContentFacet content = content(context);
    SwiftCoordinates canonical = content.canonical(SwiftPaths.coordinates(context));
    Map<String, JsonNode> versions = new LinkedHashMap<>();
    releases.fields().forEachRemaining(entry -> {
      if (SwiftVersion.isValid(entry.getKey())) {
        JsonNode problem = entry.getValue() == null ? null : entry.getValue().get(ReleaseResponses.PROBLEM);
        versions.put(entry.getKey(), problem);
      }
    });
    List<Map<String, String>> locations = SwiftHandlers.storedLocations(
        content.getAsset(canonical.getReleasesPath()).orElse(null));
    return ReleaseResponses.releases(SwiftPaths.baseUrl(context), canonical, versions, locations);
  };

  public final Handler release = context -> {
    Response response = context.proceed();
    if (!isOk(response)) {
      return response;
    }
    SwiftContentFacet content = content(context);
    SwiftCoordinates canonical = content.canonical(SwiftPaths.coordinates(context));
    Links links = ReleaseResponses.releaseLinks(SwiftPaths.baseUrl(context), canonical,
        cachedVersions(content, canonical));
    if (!links.isEmpty()) {
      response.getHeaders().replace(LINK, links.toString());
    }
    return response;
  };

  public final Handler manifest = context -> {
    SwiftCoordinates requested = SwiftPaths.coordinates(context);
    String filename = SwiftPaths.manifestFilename(context);
    String swiftVersion = SwiftPaths.swiftVersion(context);
    if (Manifests.MANIFEST.equals(filename) && swiftVersion != null) {
      Response unqualified = dispatchUnqualified(context);
      if (!unqualified.getStatus().isSuccessful()) {
        return unqualified;
      }
      SwiftCoordinates canonical = content(context).canonical(requested);
      Map<String, String> alternates = SwiftHandlers.alternates(
          content(context).getAsset(canonical.getManifestPath(Manifests.MANIFEST)).orElse(null));
      Optional<String> resolved = Manifests.resolve(alternates, swiftVersion);
      if (resolved.isEmpty()) {
        return SwiftResponses.seeOther(SwiftPaths.baseUrl(context) + canonical.getManifestPath(Manifests.MANIFEST));
      }
      filename = resolved.get();
      context.getAttributes().set(SwiftPaths.MANIFEST_FILENAME, filename);
    }
    Manifests.variantVersion(filename)
        .ifPresent(version -> context.getAttributes().set(SwiftPaths.UPSTREAM_SWIFT_VERSION, version));
    Response response = context.proceed();
    if (!isOk(response) || !(response.getPayload() instanceof Content payload)) {
      return response;
    }
    SwiftCoordinates canonical = content(context).canonical(requested);
    Map<String, String> alternates = Manifests.MANIFEST.equals(filename)
        ? SwiftHandlers.alternates(content(context).getAsset(canonical.getManifestPath(Manifests.MANIFEST)).orElse(null))
        : Map.of();
    return ReleaseResponses.manifest(SwiftPaths.baseUrl(context), canonical, filename, payload, alternates);
  };

  public final Handler archive = context -> {
    Response response = context.proceed();
    if (!isOk(response) || !(response.getPayload() instanceof Content payload)) {
      return response;
    }
    SwiftContentFacet content = content(context);
    SwiftCoordinates requested = SwiftPaths.coordinates(context);
    Optional<FluentComponent> component = content.release(requested);
    SwiftCoordinates canonical = component
        .map(release -> requested.withPackage(release.namespace(), release.name()))
        .orElse(requested);
    Map<String, Object> attributes = component.map(SwiftContentFacet::swiftAttributes).orElse(Map.of());
    return ReleaseResponses.archive(canonical, payload, (String) attributes.get(CHECKSUM),
        SwiftContentFacet.signing(attributes.get(SIGNING)));
  };

  public final Handler identifiers = context -> {
    String url = SwiftHandlers.requireUrlParameter(context);
    List<String> identifiers;
    try {
      identifiers = context.getRepository().facet(SwiftProxyFacet.class).remoteIdentifiers(url);
    }
    catch (IOException | RuntimeException e) {
      log.debug("Identifier lookup of {} at the remote of {} failed", url, context.getRepository().getName(), e);
      identifiers = List.of();
    }
    if (identifiers.isEmpty()) {
      throw SwiftProblem.notFound("no package identifiers are registered for " + url);
    }
    ObjectNode body = SwiftResponses.object();
    identifiers.forEach(body.putArray("identifiers")::add);
    return SwiftResponses.json(body);
  };

  public final Handler publish = context -> {
    throw SwiftProblem.methodNotAllowed("publishing isn't supported: repository " + context.getRepository().getName()
        + " is a proxy");
  };

  private static Response dispatchUnqualified(final Context context) throws Exception {
    Request request = new Request.Builder()
        .copy(context.getRequest())
        .action(HttpMethods.GET)
        .parameters(new Parameters())
        .build();
    Response response = context.getRepository().facet(ViewFacet.class).dispatch(request, context);
    Payload payload = response.getPayload();
    if (payload != null) {
      payload.close();
    }
    return response;
  }

  static List<String> cachedVersions(final SwiftContentFacet content, final SwiftCoordinates canonical) {
    List<String> versions = new ArrayList<>();
    Optional<FluentAsset> asset = content.getAsset(canonical.getReleasesPath());
    if (asset.isEmpty()) {
      return versions;
    }
    try (InputStream in = asset.get().download().openInputStream()) {
      JsonNode body = SwiftResponses.readQuietly(in.readAllBytes());
      JsonNode releases = body == null ? null : body.get(ReleaseResponses.RELEASES);
      if (releases instanceof ObjectNode object) {
        object.fields().forEachRemaining(entry -> {
          if (SwiftVersion.isValid(entry.getKey()) && entry.getValue().get(ReleaseResponses.PROBLEM) == null) {
            versions.add(entry.getKey());
          }
        });
      }
    }
    catch (IOException e) {
      log.debug("Unable to read the cached release list of {}", canonical, e);
    }
    return versions;
  }

  static boolean isOk(final Response response) {
    return response.getStatus().getCode() == 200 && response.getPayload() != null;
  }

  @Nullable
  static byte[] body(final Response response) throws IOException {
    if (!isOk(response) || !(response.getPayload() instanceof Content content)) {
      return null;
    }
    try (InputStream in = content.openInputStream()) {
      return in.readAllBytes();
    }
  }
}
