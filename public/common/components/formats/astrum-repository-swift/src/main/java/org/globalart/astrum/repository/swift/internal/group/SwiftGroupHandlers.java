package org.globalart.astrum.repository.swift.internal.group;

import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.TreeMap;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.swift.SwiftCoordinates;
import org.globalart.astrum.repository.swift.SwiftVersion;
import org.globalart.astrum.repository.swift.internal.Links;
import org.globalart.astrum.repository.swift.internal.ReleaseResponses;
import org.globalart.astrum.repository.swift.internal.SwiftPaths;
import org.globalart.astrum.repository.swift.internal.SwiftProblem;
import org.globalart.astrum.repository.swift.internal.SwiftResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.ViewFacet;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;

import static org.globalart.astrum.repository.swift.internal.SwiftResponses.LINK;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.LOCATION;

public final class SwiftGroupHandlers
{
  private SwiftGroupHandlers() {
  }

  @org.springframework.stereotype.Component
  public static class Releases
      extends GroupHandler
  {
    @Nonnull
    @Override
    protected Response doGet(@Nonnull final Context context, @Nonnull final DispatchedRepositories dispatched)
        throws Exception
    {
      GroupFacet group = context.getRepository().facet(GroupFacet.class);
      SwiftCoordinates requested = SwiftPaths.coordinates(context);
      Map<String, JsonNode> versions = new LinkedHashMap<>();
      List<Map<String, String>> locations = new ArrayList<>();
      SwiftCoordinates canonical = null;
      for (Response response : getAll(context, group.members(), dispatched).values()) {
        JsonNode body = SwiftResponses.readQuietly(read(response));
        if (body == null || !(body.get(ReleaseResponses.RELEASES) instanceof ObjectNode releases)) {
          continue;
        }
        if (canonical == null) {
          canonical = canonical(requested, releases);
          locations.addAll(locations(response));
        }
        releases.fields().forEachRemaining(entry -> {
          if (!SwiftVersion.isValid(entry.getKey())) {
            return;
          }
          JsonNode problem = entry.getValue() == null ? null : entry.getValue().get(ReleaseResponses.PROBLEM);
          if (!versions.containsKey(entry.getKey()) || (versions.get(entry.getKey()) != null && problem == null)) {
            versions.put(entry.getKey(), problem);
          }
        });
      }
      if (canonical == null) {
        throw SwiftProblem.notFound("package " + requested.getIdentifier() + " not found");
      }
      return ReleaseResponses.releases(SwiftPaths.baseUrl(context), canonical, versions, locations);
    }

    private static SwiftCoordinates canonical(final SwiftCoordinates requested, final ObjectNode releases) {
      for (JsonNode release : releases) {
        String url = release.path(ReleaseResponses.URL).asText(null);
        SwiftCoordinates fromUrl = fromReleaseUrl(url);
        if (fromUrl != null && fromUrl.samePackage(requested.getScope(), requested.getName())) {
          return requested.withPackage(fromUrl.getScope(), fromUrl.getName());
        }
      }
      return requested;
    }

    private static List<Map<String, String>> locations(final Response response) {
      List<Map<String, String>> locations = new ArrayList<>();
      for (Links.Link link : Links.parse(response.getHeaders().getAll(LINK))) {
        if ("canonical".equals(link.rel()) || "alternate".equals(link.rel())) {
          locations.add(Map.of(ReleaseResponses.URL, link.url(), "rel", link.rel()));
        }
      }
      return locations;
    }
  }

  @org.springframework.stereotype.Component
  public static class Identifiers
      extends GroupHandler
  {
    @Nonnull
    @Override
    protected Response doGet(@Nonnull final Context context, @Nonnull final DispatchedRepositories dispatched)
        throws Exception
    {
      String url = context.getRequest().getParameters().get("url");
      if (url == null || url.isBlank()) {
        throw SwiftProblem.badRequest("the url query parameter is required");
      }
      GroupFacet group = context.getRepository().facet(GroupFacet.class);
      Map<String, String> identifiers = new TreeMap<>();
      for (Response response : getAll(context, group.members(), dispatched).values()) {
        JsonNode body = SwiftResponses.readQuietly(read(response));
        JsonNode values = body == null ? null : body.get("identifiers");
        if (values != null && values.isArray()) {
          values.forEach(value -> {
            if (value.isTextual()) {
              identifiers.putIfAbsent(value.asText().toLowerCase(Locale.ROOT), value.asText());
            }
          });
        }
      }
      if (identifiers.isEmpty()) {
        throw SwiftProblem.notFound("no package identifiers are registered for " + url);
      }
      ObjectNode body = SwiftResponses.object();
      identifiers.values().forEach(body.putArray("identifiers")::add);
      return SwiftResponses.json(body);
    }
  }

  @org.springframework.stereotype.Component
  public static class FirstMember
      extends GroupHandler
  {
    @Nonnull
    @Override
    protected Response doGet(@Nonnull final Context context, @Nonnull final DispatchedRepositories dispatched)
        throws Exception
    {
      GroupFacet group = context.getRepository().facet(GroupFacet.class);
      for (Repository member : group.members()) {
        boolean offline = member.getConfiguration() != null && !member.getConfiguration().isOnline();
        if (offline || dispatched.contains(member)) {
          continue;
        }
        dispatched.add(member);
        Response response = member.facet(ViewFacet.class).dispatch(context.getRequest(), context);
        int status = response.getStatus().getCode();
        if (status == 200 || status == 303) {
          rewrite(response, SwiftPaths.baseUrl(context), base(member));
          return response;
        }
        close(response);
      }
      throw SwiftProblem.notFound("not found in any member of " + context.getRepository().getName());
    }
  }

  static void rewrite(final Response response, final String groupBase, final String memberBase) {
    for (String header : List.of(LINK, LOCATION)) {
      List<String> values = new ArrayList<>(response.getHeaders().getAll(header));
      if (!values.isEmpty()) {
        response.getHeaders().remove(header);
        values.forEach(value -> response.getHeaders().set(header, rewrite(value, groupBase, memberBase)));
      }
    }
  }

  static String rewrite(final String value, final String groupBase, final String memberBase) {
    return value.replace(memberBase + "/", groupBase + "/");
  }

  static String base(final Repository repository) {
    String url = repository.getUrl();
    return url.endsWith("/") ? url.substring(0, url.length() - 1) : url;
  }

  @Nullable
  static SwiftCoordinates fromReleaseUrl(@Nullable final String url) {
    if (url == null) {
      return null;
    }
    String[] segments = url.split("/");
    if (segments.length < 3) {
      return null;
    }
    String scope = segments[segments.length - 3];
    String name = segments[segments.length - 2];
    return SwiftCoordinates.isValidScope(scope) && SwiftCoordinates.isValidName(name)
        ? new SwiftCoordinates(scope, name, null)
        : null;
  }

  @Nullable
  static byte[] read(final Response response) throws IOException {
    Payload payload = response.getPayload();
    if (response.getStatus().getCode() != 200 || payload == null) {
      close(response);
      return null;
    }
    try (Payload closeable = payload; InputStream in = closeable.openInputStream()) {
      return in.readAllBytes();
    }
  }

  static void close(final Response response) throws IOException {
    if (response.getPayload() != null) {
      response.getPayload().close();
    }
  }
}
