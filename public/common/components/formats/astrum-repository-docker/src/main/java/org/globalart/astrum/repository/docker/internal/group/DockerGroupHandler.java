package org.globalart.astrum.repository.docker.internal.group;

import java.io.InputStream;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Map.Entry;
import java.util.TreeSet;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.docker.internal.v2.DockerError;
import org.globalart.astrum.repository.docker.internal.v2.DockerLocations;
import org.globalart.astrum.repository.docker.internal.v2.DockerMetadataHandler;
import org.globalart.astrum.repository.docker.internal.v2.DockerResponses;
import org.globalart.astrum.repository.docker.internal.v2.DockerRoute;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Parameters;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;

import com.fasterxml.jackson.databind.JsonNode;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier("docker")
public class DockerGroupHandler
    extends GroupHandler
{
  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    DockerRoute route = DockerRoute.get(context);
    String action = context.getRequest().getAction();
    boolean read = HttpMethods.GET.equals(action) || HttpMethods.HEAD.equals(action);
    if (!read || route.getType() == DockerRoute.Type.UPLOADS || route.getType() == DockerRoute.Type.UPLOAD) {
      return write(context);
    }
    switch (route.getType()) {
      case TAGS:
        return tags(context, route);
      case CATALOG:
        return catalog(context);
      case MANIFEST:
      case BLOB:
        return super.handle(context);
      default:
        return DockerResponses.error(DockerError.UNSUPPORTED);
    }
  }

  @Override
  protected Response notFoundResponse(final Context context) {
    DockerRoute route = DockerRoute.get(context);
    if (route.getType() == DockerRoute.Type.BLOB) {
      return DockerResponses.error(DockerError.BLOB_UNKNOWN, DockerError.BLOB_UNKNOWN.message(),
          Map.of("digest", route.getReference()));
    }
    return DockerResponses.error(DockerError.MANIFEST_UNKNOWN, DockerError.MANIFEST_UNKNOWN.message(),
        Map.of("name", DockerLocations.externalName(context, route.getName()), "reference", route.getReference()));
  }

  private static Response write(final Context context) {
    return DockerResponses.error(DockerError.UNSUPPORTED,
        "group repository " + context.getRepository().getName() + " is read-only, push to a hosted member instead");
  }

  private Response tags(final Context context, final DockerRoute route) throws Exception {
    TreeSet<String> tags = new TreeSet<>();
    boolean found = false;
    for (Entry<Repository, Response> entry : dispatchAll(context).entrySet()) {
      JsonNode body = json(entry.getValue());
      if (body == null) {
        continue;
      }
      found = true;
      for (JsonNode tag : body.path("tags")) {
        tags.add(tag.asText());
      }
    }
    return DockerMetadataHandler.tags(context, route.getName(), found ? tags : null);
  }

  private Response catalog(final Context context) throws Exception {
    TreeSet<String> names = new TreeSet<>();
    for (Entry<Repository, Response> entry : dispatchAll(context).entrySet()) {
      JsonNode body = json(entry.getValue());
      if (body == null) {
        continue;
      }
      for (JsonNode name : body.path("repositories")) {
        names.add(name.asText());
      }
    }
    return DockerMetadataHandler.catalog(context, names);
  }

  private LinkedHashMap<Repository, Response> dispatchAll(final Context context) throws Exception {
    Request original = context.getRequest();
    Parameters parameters = new Parameters();
    for (Entry<String, String> parameter : original.getParameters()) {
      if (!"n".equals(parameter.getKey()) && !"last".equals(parameter.getKey())) {
        parameters.set(parameter.getKey(), parameter.getValue());
      }
    }
    Request request = new Request.Builder()
        .copy(original)
        .parameters(parameters)
        .build();
    return getAll(request, context, context.getRepository().facet(GroupFacet.class).members(),
        new DispatchedRepositories());
  }

  private static JsonNode json(final Response response) {
    if (!response.getStatus().isSuccessful() || response.getPayload() == null) {
      return null;
    }
    try (InputStream in = response.getPayload().openInputStream()) {
      return DockerResponses.MAPPER.readTree(in);
    }
    catch (Exception e) {
      return null;
    }
  }
}
