package org.globalart.astrum.repository.nuget.internal.metadata;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.nuget.internal.NugetPaths;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetPackageMetadata.Dependency;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetPackageMetadata.DependencyGroup;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;

import static org.globalart.astrum.repository.nuget.internal.NugetPaths.lower;

public final class NugetV3Documents
{
  public static final String JSON = "application/json";

  public static final ObjectMapper MAPPER = new ObjectMapper();

  private static final String UNLISTED_PUBLISHED = "1900-01-01T00:00:00+00:00";

  private NugetV3Documents() {
  }

  public static String write(final Object value) {
    try {
      return MAPPER.writeValueAsString(value);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  public static Map<String, Object> read(final String json) throws IOException {
    return MAPPER.readValue(json, new TypeReference<LinkedHashMap<String, Object>>() { });
  }

  public static String serviceIndex(final String repositoryUrl, final boolean publish, final boolean legacyFeed) {
    List<Map<String, Object>> resources = new ArrayList<>();
    String content = repositoryUrl + NugetPaths.CONTENT;
    String registration = repositoryUrl + NugetPaths.REGISTRATION;
    String query = repositoryUrl + NugetPaths.QUERY;
    String autocomplete = repositoryUrl + NugetPaths.AUTOCOMPLETE;
    resources.add(resource(content, "PackageBaseAddress/3.0.0", "Base URL of the flat container"));
    if (publish) {
      resources.add(resource(repositoryUrl + "/", "PackagePublish/2.0.0", "Push and delete packages"));
    }
    for (String type : new String[]{"RegistrationsBaseUrl", "RegistrationsBaseUrl/3.0.0-beta",
        "RegistrationsBaseUrl/3.0.0-rc", "RegistrationsBaseUrl/3.4.0", "RegistrationsBaseUrl/3.6.0",
        "RegistrationsBaseUrl/Versioned"}) {
      resources.add(resource(registration, type, "Package registrations"));
    }
    for (String type : new String[]{"SearchQueryService", "SearchQueryService/3.0.0-beta",
        "SearchQueryService/3.0.0-rc", "SearchQueryService/3.5.0"}) {
      resources.add(resource(query, type, "Search packages"));
    }
    for (String type : new String[]{"SearchAutocompleteService", "SearchAutocompleteService/3.0.0-beta",
        "SearchAutocompleteService/3.0.0-rc", "SearchAutocompleteService/3.5.0"}) {
      resources.add(resource(autocomplete, type, "Autocomplete package ids and versions"));
    }
    if (legacyFeed) {
      resources.add(resource(repositoryUrl + "/", "LegacyGallery/2.0.0", "NuGet V2 feed"));
    }
    Map<String, Object> index = new LinkedHashMap<>();
    index.put("version", "3.0.0");
    index.put("resources", resources);
    return write(index);
  }

  private static Map<String, Object> resource(final String id, final String type, final String comment) {
    Map<String, Object> resource = new LinkedHashMap<>();
    resource.put("@id", id);
    resource.put("@type", type);
    resource.put("comment", comment);
    return resource;
  }

  public static String versions(final Collection<NugetVersion> versions) {
    List<String> lower = versions.stream().sorted().distinct().map(NugetVersion::toLowerString).toList();
    return write(Map.of("versions", lower));
  }

  public static List<NugetPackageMetadata> sorted(final Collection<NugetPackageMetadata> packages) {
    List<NugetPackageMetadata> sorted = new ArrayList<>(packages);
    sorted.sort(Comparator.comparing(NugetPackageMetadata::getVersion));
    return sorted;
  }

  public static String registrationIndex(
      final String repositoryUrl,
      final String id,
      final Collection<NugetPackageMetadata> packages)
  {
    String indexUrl = NugetPaths.registrationIndexUrl(repositoryUrl, id);
    List<NugetPackageMetadata> sorted = sorted(packages);
    List<Map<String, Object>> leaves = new ArrayList<>();
    for (NugetPackageMetadata metadata : sorted) {
      leaves.add(registrationLeafInline(repositoryUrl, metadata));
    }
    String lowerVersion = lower(sorted.get(0).getVersion().toNormalizedString());
    String upperVersion = lower(sorted.get(sorted.size() - 1).getVersion().toNormalizedString());
    Map<String, Object> page = new LinkedHashMap<>();
    page.put("@id", indexUrl + "#page/" + lowerVersion + "/" + upperVersion);
    page.put("@type", "catalog:CatalogPage");
    page.put("count", leaves.size());
    page.put("items", leaves);
    page.put("lower", lowerVersion);
    page.put("parent", indexUrl);
    page.put("upper", upperVersion);

    Map<String, Object> index = new LinkedHashMap<>();
    index.put("@id", indexUrl);
    index.put("@type", List.of("catalog:CatalogRoot", "PackageRegistration", "catalog:Permalink"));
    index.put("count", 1);
    index.put("items", List.of(page));
    return write(index);
  }

  private static Map<String, Object> registrationLeafInline(
      final String repositoryUrl,
      final NugetPackageMetadata metadata)
  {
    String version = metadata.getVersion().toNormalizedString();
    Map<String, Object> leaf = new LinkedHashMap<>();
    leaf.put("@id", NugetPaths.registrationLeafUrl(repositoryUrl, metadata.getId(), version));
    leaf.put("@type", "Package");
    leaf.put("catalogEntry", catalogEntryMap(repositoryUrl, metadata));
    leaf.put("packageContent", NugetPaths.contentUrl(repositoryUrl, metadata.getId(), version));
    leaf.put("registration", NugetPaths.registrationIndexUrl(repositoryUrl, metadata.getId()));
    return leaf;
  }

  public static String registrationLeaf(final String repositoryUrl, final NugetPackageMetadata metadata) {
    String version = metadata.getVersion().toNormalizedString();
    Map<String, Object> leaf = new LinkedHashMap<>();
    leaf.put("@id", NugetPaths.registrationLeafUrl(repositoryUrl, metadata.getId(), version));
    leaf.put("@type", List.of("Package", "http://schema.nuget.org/catalog#Permalink"));
    leaf.put("catalogEntry", NugetPaths.catalogUrl(repositoryUrl, metadata.getId(), version));
    leaf.put("listed", metadata.isListed());
    leaf.put("packageContent", NugetPaths.contentUrl(repositoryUrl, metadata.getId(), version));
    leaf.put("published", published(metadata));
    leaf.put("registration", NugetPaths.registrationIndexUrl(repositoryUrl, metadata.getId()));
    return write(leaf);
  }

  public static String catalogEntry(final String repositoryUrl, final NugetPackageMetadata metadata) {
    return write(catalogEntryMap(repositoryUrl, metadata));
  }

  private static String published(final NugetPackageMetadata metadata) {
    if (!metadata.isListed()) {
      return UNLISTED_PUBLISHED;
    }
    String published = metadata.text("published");
    return published.isEmpty() ? UNLISTED_PUBLISHED : published;
  }

  private static Map<String, Object> catalogEntryMap(final String repositoryUrl, final NugetPackageMetadata metadata) {
    String version = metadata.getVersion().toNormalizedString();
    String catalogUrl = NugetPaths.catalogUrl(repositoryUrl, metadata.getId(), version);
    Map<String, Object> entry = new LinkedHashMap<>();
    entry.put("@id", catalogUrl);
    entry.put("@type", "PackageDetails");
    entry.put("authors", metadata.text("authors"));
    List<Map<String, Object>> groups = new ArrayList<>();
    for (DependencyGroup group : metadata.getDependencyGroups()) {
      String framework = group.getTargetFramework();
      String groupId = catalogUrl + "#dependencygroup" + (framework == null ? "" : "/" + lower(framework));
      Map<String, Object> groupMap = new LinkedHashMap<>();
      groupMap.put("@id", groupId);
      groupMap.put("@type", "PackageDependencyGroup");
      List<Map<String, Object>> dependencies = new ArrayList<>();
      for (Dependency dependency : group.getDependencies()) {
        Map<String, Object> dependencyMap = new LinkedHashMap<>();
        dependencyMap.put("@id", groupId + "/" + lower(dependency.getId()));
        dependencyMap.put("@type", "PackageDependency");
        dependencyMap.put("id", dependency.getId());
        dependencyMap.put("range", normalizeRange(dependency.getRange()));
        dependencyMap.put("registration", NugetPaths.registrationIndexUrl(repositoryUrl, dependency.getId()));
        dependencies.add(dependencyMap);
      }
      groupMap.put("dependencies", dependencies);
      if (framework != null) {
        groupMap.put("targetFramework", framework);
      }
      groups.add(groupMap);
    }
    entry.put("dependencyGroups", groups);
    entry.put("description", metadata.text("description"));
    entry.put("iconUrl", metadata.text("icon_url"));
    entry.put("id", metadata.getId());
    entry.put("language", metadata.text("language"));
    entry.put("licenseExpression", metadata.text("license_expression"));
    entry.put("licenseUrl", metadata.text("license_url"));
    entry.put("listed", metadata.isListed());
    entry.put("minClientVersion", metadata.text("min_client_version"));
    entry.put("packageContent", NugetPaths.contentUrl(repositoryUrl, metadata.getId(), version));
    entry.put("projectUrl", metadata.text("project_url"));
    entry.put("published", published(metadata));
    entry.put("requireLicenseAcceptance", metadata.isRequireLicenseAcceptance());
    entry.put("summary", metadata.text("summary"));
    entry.put("tags", metadata.getTagList());
    entry.put("title", metadata.text("title"));
    entry.put("version", metadata.getVersion().toFullString());
    return entry;
  }

  public static String normalizeRange(final String range) {
    String value = range == null ? "" : range.trim();
    if (value.isEmpty()) {
      return "(, )";
    }
    char first = value.charAt(0);
    if (first != '[' && first != '(') {
      return NugetVersion.normalize(value).map(version -> "[" + version + ", )").orElse(value);
    }
    char last = value.charAt(value.length() - 1);
    if (value.length() < 2 || (last != ']' && last != ')')) {
      return value;
    }
    String inner = value.substring(1, value.length() - 1);
    int comma = inner.indexOf(',');
    if (comma < 0) {
      String exact = normalizeOrSelf(inner.trim());
      return "[" + exact + ", " + exact + "]";
    }
    String lowerBound = normalizeOrSelf(inner.substring(0, comma).trim());
    String upperBound = normalizeOrSelf(inner.substring(comma + 1).trim());
    return first + lowerBound + ", " + upperBound + last;
  }

  private static String normalizeOrSelf(final String version) {
    return version.isEmpty() ? version : NugetVersion.normalize(version).orElse(version);
  }

  public static Map<String, Object> searchHit(
      final String repositoryUrl,
      final NugetPackageMetadata latest,
      final List<NugetPackageMetadata> versions)
  {
    Map<String, Object> hit = new LinkedHashMap<>();
    String registration = NugetPaths.registrationIndexUrl(repositoryUrl, latest.getId());
    hit.put("@id", registration);
    hit.put("@type", "Package");
    hit.put("registration", registration);
    hit.put("id", latest.getId());
    hit.put("version", latest.getVersion().toFullString());
    hit.put("description", latest.text("description"));
    hit.put("summary", latest.text("summary"));
    hit.put("title", latest.text("title"));
    hit.put("iconUrl", latest.text("icon_url"));
    hit.put("licenseUrl", latest.text("license_url"));
    hit.put("projectUrl", latest.text("project_url"));
    hit.put("tags", latest.getTagList());
    hit.put("authors", latest.getAuthorList());
    hit.put("owners", latest.getOwnerList());
    hit.put("totalDownloads", 0);
    hit.put("verified", false);
    hit.put("packageTypes", List.of(Map.of("name", "Dependency")));
    List<Map<String, Object>> versionList = new ArrayList<>();
    for (NugetPackageMetadata version : sorted(versions)) {
      Map<String, Object> versionMap = new LinkedHashMap<>();
      versionMap.put("version", version.getVersion().toFullString());
      versionMap.put("downloads", 0);
      versionMap.put("@id", NugetPaths.registrationLeafUrl(repositoryUrl, version.getId(),
          version.getVersion().toNormalizedString()));
      versionList.add(versionMap);
    }
    hit.put("versions", versionList);
    return hit;
  }

  public static String searchResult(final long totalHits, final List<Map<String, Object>> data) {
    Map<String, Object> result = new LinkedHashMap<>();
    result.put("totalHits", totalHits);
    result.put("data", data);
    return write(result);
  }

  public static String autocompleteResult(final long totalHits, final List<String> data) {
    Map<String, Object> result = new LinkedHashMap<>();
    result.put("totalHits", totalHits);
    result.put("data", data);
    return write(result);
  }
}
