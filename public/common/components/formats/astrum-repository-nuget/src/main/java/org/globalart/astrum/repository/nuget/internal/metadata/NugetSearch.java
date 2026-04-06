package org.globalart.astrum.repository.nuget.internal.metadata;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Parameters;

public final class NugetSearch
{
  public static final int DEFAULT_TAKE = 20;

  public static final int MAX_TAKE = 1000;

  private final List<String> terms = new ArrayList<>();

  private final Map<String, String> fields = new LinkedHashMap<>();

  private final int skip;

  private final int take;

  private final boolean prerelease;

  private final boolean semVer2;

  public record Hit(NugetPackageMetadata latest, List<NugetPackageMetadata> versions)
  {
  }

  public record Result(long totalHits, List<Hit> hits)
  {
  }

  public NugetSearch(
      @Nullable final String query,
      final int skip,
      final int take,
      final boolean prerelease,
      final boolean semVer2)
  {
    this.skip = Math.max(0, skip);
    this.take = Math.max(0, Math.min(MAX_TAKE, take));
    this.prerelease = prerelease;
    this.semVer2 = semVer2;
    if (query != null) {
      for (String token : query.trim().split("\\s+")) {
        if (token.isEmpty()) {
          continue;
        }
        int colon = token.indexOf(':');
        if (colon > 0 && colon < token.length() - 1) {
          fields.put(token.substring(0, colon).toLowerCase(Locale.ROOT),
              stripQuotes(token.substring(colon + 1)).toLowerCase(Locale.ROOT));
        }
        else {
          terms.add(stripQuotes(token).toLowerCase(Locale.ROOT));
        }
      }
    }
  }

  public static NugetSearch fromV3(final Parameters parameters) {
    return new NugetSearch(
        parameters.get("q"),
        intParameter(parameters.get("skip"), 0),
        intParameter(parameters.get("take"), DEFAULT_TAKE),
        Boolean.parseBoolean(parameters.get("prerelease")),
        isSemVer2(parameters.get("semVerLevel")));
  }

  public static boolean isSemVer2(@Nullable final String semVerLevel) {
    return semVerLevel != null && NugetVersion.parse(semVerLevel)
        .map(version -> version.compareTo(NugetVersion.parse("2.0.0").orElseThrow()) >= 0)
        .orElse(false);
  }

  public static int intParameter(@Nullable final String value, final int defaultValue) {
    if (value == null || value.isBlank()) {
      return defaultValue;
    }
    try {
      return Integer.parseInt(value.trim());
    }
    catch (NumberFormatException e) {
      return defaultValue;
    }
  }

  private static String stripQuotes(final String value) {
    String result = value;
    if (result.length() >= 2 && (result.startsWith("\"") && result.endsWith("\"")
        || result.startsWith("'") && result.endsWith("'"))) {
      result = result.substring(1, result.length() - 1);
    }
    return result;
  }

  public int getSkip() {
    return skip;
  }

  public int getTake() {
    return take;
  }

  public boolean isPrerelease() {
    return prerelease;
  }

  public boolean isSemVer2() {
    return semVer2;
  }

  public boolean accepts(final NugetPackageMetadata metadata) {
    if (!metadata.isListed()) {
      return false;
    }
    if (!prerelease && metadata.getVersion().isPrerelease()) {
      return false;
    }
    return semVer2 || !metadata.isSemVer2();
  }

  public Result search(final Collection<NugetPackageMetadata> packages) {
    Map<String, List<NugetPackageMetadata>> byId = new LinkedHashMap<>();
    for (NugetPackageMetadata metadata : packages) {
      if (accepts(metadata)) {
        byId.computeIfAbsent(metadata.getLowerId(), key -> new ArrayList<>()).add(metadata);
      }
    }
    List<Hit> hits = new ArrayList<>();
    for (List<NugetPackageMetadata> versions : byId.values()) {
      NugetPackageMetadata latest = versions.stream().max(Comparator.comparing(NugetPackageMetadata::getVersion))
          .orElseThrow();
      if (matches(latest)) {
        hits.add(new Hit(latest, versions));
      }
    }
    hits.sort(Comparator.comparingInt((Hit hit) -> rank(hit.latest()))
        .thenComparing(hit -> hit.latest().getLowerId()));
    int from = Math.min(skip, hits.size());
    int to = Math.min(hits.size(), from + take);
    return new Result(hits.size(), new ArrayList<>(hits.subList(from, to)));
  }

  private int rank(final NugetPackageMetadata metadata) {
    String id = metadata.getLowerId();
    for (String term : terms) {
      if (id.equals(term)) {
        return 0;
      }
    }
    for (String term : terms) {
      if (id.startsWith(term)) {
        return 1;
      }
    }
    return 2;
  }

  public boolean matches(final NugetPackageMetadata metadata) {
    String id = metadata.getLowerId();
    for (Map.Entry<String, String> field : fields.entrySet()) {
      String value = field.getValue();
      boolean matched = switch (field.getKey()) {
        case "id" -> id.contains(value);
        case "packageid" -> id.equals(value);
        case "version" -> metadata.getVersion().toLowerString().equals(value);
        case "title" -> lower(metadata.text("title")).contains(value);
        case "tags", "tag" -> metadata.getTagList().stream().anyMatch(tag -> lower(tag).equals(value));
        case "author", "authors" -> lower(metadata.text("authors")).contains(value);
        case "owner", "owners" -> lower(metadata.text("owners")).contains(value);
        case "description" -> lower(metadata.text("description")).contains(value);
        case "summary" -> lower(metadata.text("summary")).contains(value);
        default -> anyField(metadata, field.getKey() + ":" + value);
      };
      if (!matched) {
        return false;
      }
    }
    for (String term : terms) {
      if (!anyField(metadata, term)) {
        return false;
      }
    }
    return true;
  }

  private static boolean anyField(final NugetPackageMetadata metadata, final String term) {
    return metadata.getLowerId().contains(term)
        || lower(metadata.text("title")).contains(term)
        || lower(metadata.text("tags")).contains(term)
        || lower(metadata.text("description")).contains(term)
        || lower(metadata.text("summary")).contains(term)
        || lower(metadata.text("authors")).contains(term);
  }

  private static String lower(final String value) {
    return value.toLowerCase(Locale.ROOT);
  }
}
