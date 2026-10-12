package org.globalart.astrum.repository.swift.internal.hosted;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.swift.SwiftCoordinates;
import org.globalart.astrum.repository.swift.internal.Manifests;
import org.globalart.astrum.repository.swift.internal.PublishRequest;
import org.globalart.astrum.repository.swift.internal.ReleaseMetadata;
import org.globalart.astrum.repository.swift.internal.ReleaseMetadata.Signing;
import org.globalart.astrum.repository.swift.internal.ReleaseResponses;
import org.globalart.astrum.repository.swift.internal.SwiftContentFacet;
import org.globalart.astrum.repository.swift.internal.SwiftHandlers;
import org.globalart.astrum.repository.swift.internal.SwiftPaths;
import org.globalart.astrum.repository.swift.internal.SwiftProblem;
import org.globalart.astrum.repository.swift.internal.SwiftResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.swift.internal.SwiftContentFacet.CHECKSUM;
import static org.globalart.astrum.repository.swift.internal.SwiftContentFacet.PUBLISHED_AT;
import static org.globalart.astrum.repository.swift.internal.SwiftContentFacet.SIGNING;
import static org.globalart.astrum.repository.swift.internal.SwiftContentFacet.swiftAttributes;
import static org.globalart.astrum.repository.swift.internal.SwiftHandlers.content;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.APPLICATION_JSON;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.APPLICATION_ZIP;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.LOCATION;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.SIGNATURE_FORMAT;

@Component
public class SwiftHostedHandlers
{
  private static final Logger log = LoggerFactory.getLogger(SwiftHostedHandlers.class);

  public final Handler releases = context -> {
    SwiftCoordinates requested = SwiftPaths.coordinates(context);
    List<FluentComponent> releases = content(context).releases(requested);
    if (releases.isEmpty()) {
      throw SwiftProblem.notFound("package " + requested.getIdentifier() + " not found");
    }
    SwiftCoordinates canonical = requested.withPackage(releases.get(0).namespace(), releases.get(0).name());
    Map<String, JsonNode> versions = new LinkedHashMap<>();
    releases.forEach(release -> versions.put(release.version(), null));
    return ReleaseResponses.releases(SwiftPaths.baseUrl(context), canonical, versions,
        SwiftHandlers.locations(releases.get(0)));
  };

  public final Handler release = context -> {
    SwiftCoordinates requested = SwiftPaths.coordinates(context);
    FluentComponent component = requireRelease(context, requested);
    SwiftCoordinates canonical = canonical(requested, component);
    Map<String, Object> attributes = swiftAttributes(component);
    ObjectNode body = ReleaseMetadata.release(canonical, (String) attributes.get(CHECKSUM),
        SwiftContentFacet.signing(attributes.get(SIGNING)), SwiftContentFacet.metadata(component),
        (String) attributes.get(PUBLISHED_AT));
    List<String> versions = content(context).releases(requested).stream().map(FluentComponent::version).toList();
    return ReleaseResponses.release(SwiftPaths.baseUrl(context), canonical,
        SwiftResponses.content(SwiftResponses.bytes(body), APPLICATION_JSON), versions);
  };

  public final Handler manifest = context -> {
    SwiftCoordinates requested = SwiftPaths.coordinates(context);
    SwiftCoordinates canonical = canonical(requested, requireRelease(context, requested));
    SwiftContentFacet content = content(context);
    FluentAsset unqualified = content.getAsset(canonical.getManifestPath(Manifests.MANIFEST))
        .orElseThrow(() -> SwiftProblem.notFound("manifest of " + canonical + " not found"));
    Map<String, String> alternates = SwiftHandlers.alternates(unqualified);
    String filename = SwiftPaths.manifestFilename(context);
    String swiftVersion = SwiftPaths.swiftVersion(context);
    if (Manifests.MANIFEST.equals(filename) && swiftVersion != null) {
      String resolved = Manifests.resolve(alternates, swiftVersion).orElse(null);
      if (resolved == null) {
        return SwiftResponses.seeOther(SwiftPaths.baseUrl(context) + canonical.getManifestPath(Manifests.MANIFEST));
      }
      filename = resolved;
    }
    FluentAsset asset = Manifests.MANIFEST.equals(filename)
        ? unqualified
        : content.getAsset(canonical.getManifestPath(filename))
            .orElseThrow(() -> SwiftProblem.notFound("manifest " + SwiftPaths.manifestFilename(context) + " of "
                + canonical + " not found"));
    return ReleaseResponses.manifest(SwiftPaths.baseUrl(context), canonical, filename, asset.download(), alternates);
  };

  public final Handler archive = context -> {
    SwiftCoordinates requested = SwiftPaths.coordinates(context);
    FluentComponent component = requireRelease(context, requested);
    SwiftCoordinates canonical = canonical(requested, component);
    FluentAsset asset = content(context).getAsset(canonical.getArchivePath())
        .orElseThrow(() -> SwiftProblem.notFound("source archive of " + canonical + " not found"));
    Map<String, Object> attributes = swiftAttributes(component);
    return ReleaseResponses.archive(canonical, asset.download(), (String) attributes.get(CHECKSUM),
        SwiftContentFacet.signing(attributes.get(SIGNING)));
  };

  public final Handler identifiers = context -> {
    String url = SwiftHandlers.requireUrlParameter(context);
    List<String> identifiers = content(context).identifiers(url);
    if (identifiers.isEmpty()) {
      throw SwiftProblem.notFound("no package identifiers are registered for " + url);
    }
    ObjectNode body = SwiftResponses.object();
    identifiers.forEach(body.putArray("identifiers")::add);
    return SwiftResponses.json(body);
  };

  public final Handler publish = context -> {
    SwiftCoordinates requested = SwiftPaths.coordinates(context);
    Request request = context.getRequest();
    if (!request.isMultipart()) {
      throw SwiftProblem.badRequest("a package release must be published as multipart/form-data");
    }
    String signatureFormat = SwiftPaths.header(context, SIGNATURE_FORMAT);
    SwiftContentFacet content = content(context);
    try (PublishRequest publish = PublishRequest.read(request.getMultiparts(), signatureFormat,
        in -> content.ingest(in, APPLICATION_ZIP))) {
      Signing signing = publish.getArchiveSignature() == null ? null
          : new Signing(publish.getArchiveSignature(), signatureFormat);
      Signing metadataSigning = publish.getMetadataSignature() == null ? null
          : new Signing(publish.getMetadataSignature(), signatureFormat);
      FluentComponent component = content.publish(requested, publish.getArchive(), publish.getMetadata(), signing,
          metadataSigning);
      SwiftCoordinates canonical = canonical(requested, component);
      log.info("Published Swift package release {} to {}", canonical, context.getRepository().getName());
      Response response = SwiftResponses.empty(201);
      response.getHeaders().set(LOCATION, SwiftPaths.baseUrl(context) + canonical.getReleasePath());
      return response;
    }
  };

  static FluentComponent requireRelease(final Context context, final SwiftCoordinates requested) {
    return content(context).release(requested)
        .filter(component -> swiftAttributes(component).get(CHECKSUM) != null)
        .orElseThrow(() -> SwiftProblem.notFound("release " + requested.getVersion() + " of "
            + requested.getIdentifier() + " not found"));
  }

  static SwiftCoordinates canonical(final SwiftCoordinates requested, final FluentComponent component) {
    return requested.withPackage(component.namespace(), component.name());
  }
}
