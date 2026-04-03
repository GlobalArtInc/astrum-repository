package org.globalart.astrum.repository.nuget.internal;

import java.util.Locale;
import java.util.Optional;

public final class NugetPaths
{
  public static final String SERVICE_INDEX = "/index.json";

  public static final String CONTENT = "/v3/content/";

  public static final String REGISTRATION = "/v3/registration/";

  public static final String CATALOG = "/v3/catalog/";

  public static final String QUERY = "/v3/query";

  public static final String AUTOCOMPLETE = "/v3/autocomplete";

  public static final String UPSTREAM_SERVICE_INDEX = "/.nuget/upstream-index.json";

  private NugetPaths() {
  }

  public static String lower(final String value) {
    return value.toLowerCase(Locale.ROOT);
  }

  public static Optional<String[]> packageCoordinates(final String path) {
    String[] segments = path.substring(1).split("/");
    if (path.startsWith(CONTENT) && segments.length == 5) {
      return Optional.of(new String[]{segments[2], segments[3]});
    }
    if (path.startsWith("/api/v2/package/") && segments.length == 5) {
      return Optional.of(new String[]{segments[3], segments[4]});
    }
    if (path.startsWith("/package/") && segments.length == 3) {
      return Optional.of(new String[]{segments[1], segments[2]});
    }
    if (segments.length == 3 && lower(segments[2]).endsWith(".nupkg")) {
      return Optional.of(new String[]{segments[0], segments[1]});
    }
    return Optional.empty();
  }

  public static String packagePath(final String id, final String version) {
    String lowerId = lower(id);
    String lowerVersion = lower(version);
    return "/" + lowerId + "/" + lowerVersion + "/" + lowerId + "." + lowerVersion + ".nupkg";
  }

  public static String packageFileName(final String id, final String version) {
    return lower(id) + "." + lower(version) + ".nupkg";
  }

  public static String nuspecFileName(final String id) {
    return lower(id) + ".nuspec";
  }

  public static String contentUrl(final String repositoryUrl, final String id, final String version) {
    String lowerId = lower(id);
    String lowerVersion = lower(version);
    return repositoryUrl + CONTENT + lowerId + "/" + lowerVersion + "/" + lowerId + "." + lowerVersion + ".nupkg";
  }

  public static String registrationIndexUrl(final String repositoryUrl, final String id) {
    return repositoryUrl + REGISTRATION + lower(id) + "/index.json";
  }

  public static String registrationLeafUrl(final String repositoryUrl, final String id, final String version) {
    return repositoryUrl + REGISTRATION + lower(id) + "/" + lower(version) + ".json";
  }

  public static String catalogUrl(final String repositoryUrl, final String id, final String version) {
    return repositoryUrl + CATALOG + lower(id) + "/" + lower(version) + ".json";
  }
}
