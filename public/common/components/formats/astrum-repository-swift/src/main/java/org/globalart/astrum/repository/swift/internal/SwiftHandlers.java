package org.globalart.astrum.repository.swift.internal;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.view.Context;

public final class SwiftHandlers
{
  public static final String URL_PARAMETER = "url";

  private SwiftHandlers() {
  }

  public static SwiftContentFacet content(final Context context) {
    return context.getRepository().facet(SwiftContentFacet.class);
  }

  public static String requireUrlParameter(final Context context) {
    String url = context.getRequest().getParameters().get(URL_PARAMETER);
    if (url == null || url.isBlank()) {
      throw SwiftProblem.badRequest("the url query parameter is required");
    }
    return url.strip();
  }

  @SuppressWarnings("unchecked")
  public static Map<String, String> alternates(@Nullable final FluentAsset manifest) {
    Map<String, String> alternates = new LinkedHashMap<>();
    if (manifest == null) {
      return alternates;
    }
    Object value = SwiftContentFacet.swiftAttributes(manifest).get(SwiftContentFacet.ALTERNATES);
    if (value instanceof Map<?, ?> map) {
      map.forEach((filename, toolsVersion) -> {
        if (filename instanceof String name && Manifests.isVariant(name)) {
          alternates.put(name, toolsVersion instanceof String text ? text : "");
        }
      });
    }
    return alternates;
  }

  public static List<Map<String, String>> locations(@Nullable final FluentComponent latest) {
    List<Map<String, String>> locations = new ArrayList<>();
    if (latest == null) {
      return locations;
    }
    Object urls = SwiftContentFacet.swiftAttributes(latest).get(SwiftContentFacet.REPOSITORY_URLS);
    if (urls instanceof List<?> list) {
      for (Object url : list) {
        if (url instanceof String text && Links.isLinkable(text)) {
          locations.add(Map.of(ReleaseResponses.URL, text, "rel", locations.isEmpty() ? "canonical" : "alternate"));
        }
      }
    }
    return locations;
  }

  @SuppressWarnings("unchecked")
  public static List<Map<String, String>> storedLocations(@Nullable final FluentAsset releases) {
    List<Map<String, String>> locations = new ArrayList<>();
    if (releases == null) {
      return locations;
    }
    Object value = SwiftContentFacet.swiftAttributes(releases).get(SwiftContentFacet.LOCATIONS);
    if (value instanceof List<?> list) {
      for (Object item : list) {
        if (item instanceof Map<?, ?> map && map.get(ReleaseResponses.URL) instanceof String url
            && map.get("rel") instanceof String rel) {
          locations.add(Map.of(ReleaseResponses.URL, url, "rel", rel));
        }
      }
    }
    return locations;
  }
}
