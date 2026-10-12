package org.globalart.astrum.repository.docker.internal.v2;

import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.docker.DockerContentFacet;
import org.globalart.astrum.repository.docker.internal.security.DockerAuth;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class DockerMetadataHandler
    implements Handler
{
  private final DockerAuth dockerAuth;

  private final long maxManifestSize;

  @Autowired
  public DockerMetadataHandler(
      final DockerAuth dockerAuth,
      @Value("${astrum.docker.manifest.maxSize:4194304}") final long maxManifestSize)
  {
    this.dockerAuth = checkNotNull(dockerAuth);
    this.maxManifestSize = maxManifestSize;
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
    if (!DockerDigest.isDigest(route.getReference())) {
      return invalidReferrersDigest(route);
    }
    DockerContentFacet content = context.getRepository().facet(DockerContentFacet.class);
    return DockerReferrers.response(context, route.getName(), route.getReference(),
        DockerReferrers.local(content, route.getName(), route.getReference()), maxManifestSize);
  }

  public static Response invalidReferrersDigest(final DockerRoute route) {
    return DockerResponses.error(DockerError.DIGEST_INVALID, "invalid digest " + route.getReference(),
        Map.of("digest", route.getReference()));
  }
}
