package org.globalart.astrum.repository.docker.internal.v2;

import java.io.IOException;
import java.util.ArrayList;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.docker.DockerContentFacet;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.docker.internal.security.DockerAuth;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import com.fasterxml.jackson.databind.JsonNode;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class DockerMetadataHandler
    implements Handler
{
  private static final Logger log = LoggerFactory.getLogger(DockerMetadataHandler.class);

  private final DockerAuth dockerAuth;

  @Autowired
  public DockerMetadataHandler(final DockerAuth dockerAuth) {
    this.dockerAuth = checkNotNull(dockerAuth);
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    DockerRoute route = DockerRoute.get(context);
    switch (route.getType()) {
      case BASE:
        return base(context);
      case CATALOG:
        return catalog(context, context.getRepository().facet(DockerContentFacet.class).names());
      case TAGS:
        return tags(context, route.getName(), localTags(context, route.getName()));
      case REFERRERS:
        return referrers(context, route);
      default:
        return DockerResponses.error(DockerError.UNSUPPORTED);
    }
  }

  private Response base(final Context context) {
    if (DockerAuth.isAnonymous() && !DockerAuth.isTokenAuthenticated(context.getRequest().getAttributes())) {
      return dockerAuth.unauthorized(context.getRepository(), context.getRequest().getAttributes(), null);
    }
    return DockerResponses.jsonResponse(200, Map.of());
  }

  @Nullable
  public static List<String> localTags(final Context context, final String name) {
    DockerContentFacet content = context.getRepository().facet(DockerContentFacet.class);
    List<String> tags = content.tags(name);
    if (tags.isEmpty() && !content.hasName(name)) {
      return null;
    }
    return tags;
  }

  public static Response tags(final Context context, final String name, @Nullable final Collection<String> tags) {
    if (tags == null) {
      return DockerResponses.error(DockerError.NAME_UNKNOWN, DockerError.NAME_UNKNOWN.message(),
          Map.of("name", DockerLocations.externalName(context, name)));
    }
    DockerPagination.Page page = DockerPagination.page(tags, context.getRequest().getParameters(),
        DockerLocations.tags(context, name));
    Map<String, Object> body = new LinkedHashMap<>();
    body.put("name", DockerLocations.externalName(context, name));
    body.put("tags", page.getItems());
    Response.Builder response = DockerResponses.success(200).payload(DockerResponses.json(body));
    if (page.getNextLink() != null) {
      response.header("Link", page.getNextLink());
    }
    return response.build();
  }

  public static Response catalog(final Context context, final Collection<String> names) {
    DockerPagination.Page page = DockerPagination.page(names, context.getRequest().getParameters(),
        DockerLocations.catalog(context));
    Response.Builder response = DockerResponses.success(200)
        .payload(DockerResponses.json(Map.of("repositories", page.getItems())));
    if (page.getNextLink() != null) {
      response.header("Link", page.getNextLink());
    }
    return response.build();
  }

  private Response referrers(final Context context, final DockerRoute route) {
    DockerContentFacet content = context.getRepository().facet(DockerContentFacet.class);
    String artifactType = context.getRequest().getParameters().get("artifactType");
    List<Map<String, Object>> descriptors = new ArrayList<>();
    for (FluentAsset asset : content.referrers(route.getName(), route.getReference())) {
      Map<String, Object> descriptor = descriptor(asset);
      if (artifactType != null && !artifactType.equals(descriptor.get("artifactType"))) {
        continue;
      }
      descriptors.add(descriptor);
    }
    Map<String, Object> body = new LinkedHashMap<>();
    body.put("schemaVersion", 2);
    body.put("mediaType", DockerManifest.OCI_INDEX);
    body.put("manifests", descriptors);
    Response.Builder response = DockerResponses.success(200)
        .payload(DockerResponses.json(body, DockerManifest.OCI_INDEX));
    if (artifactType != null) {
      response.header("OCI-Filters-Applied", "artifactType");
    }
    return response.build();
  }

  private static Map<String, Object> descriptor(final FluentAsset asset) {
    Map<String, Object> descriptor = new LinkedHashMap<>();
    descriptor.put("mediaType", DockerContentFacet.mediaTypeOf(asset));
    descriptor.put("digest", DockerContentFacet.digestOf(asset));
    descriptor.put("size", asset.blob().map(blob -> blob.blobSize()).orElse(0L));
    Object artifactType = asset.attributes(DockerFormat.NAME).get(DockerContentFacet.ATTR_ARTIFACT_TYPE);
    if (artifactType != null) {
      descriptor.put("artifactType", artifactType.toString());
    }
    try (var in = asset.download().openInputStream()) {
      JsonNode annotations = DockerResponses.MAPPER.readTree(in).path("annotations");
      if (annotations.isObject() && annotations.size() > 0) {
        descriptor.put("annotations", annotations);
      }
    }
    catch (IOException | RuntimeException e) {
      log.debug("Unable to read annotations of {}", asset.path(), e);
    }
    return descriptor;
  }
}
