package org.globalart.astrum.repository.nuget.internal.metadata;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;

import javax.annotation.Nullable;

import static com.google.common.base.Strings.emptyToNull;
import static com.google.common.base.Strings.nullToEmpty;

public class NugetPackageMetadata
{
  public static final String ID = "id";

  public static final String VERSION = "version";

  public static final String NORMALIZED_VERSION = "normalized_version";

  public static final String TAGS = "tags";

  private static final String[] TEXT_FIELDS = {
      "title", "authors", "owners", "description", "summary", "release_notes", "copyright", "language",
      TAGS, "project_url", "icon_url", "license_url", "license_expression", "min_client_version", "published",
      "package_hash", "package_hash_algorithm"
  };

  private String id;

  private NugetVersion version;

  private final Map<String, String> text = new LinkedHashMap<>();

  private boolean requireLicenseAcceptance;

  private boolean developmentDependency;

  private boolean listed = true;

  private long packageSize;

  private List<DependencyGroup> dependencyGroups = new ArrayList<>();

  public static class Dependency
  {
    private final String id;

    private final String range;

    public Dependency(final String id, final String range) {
      this.id = id;
      this.range = range;
    }

    public String getId() {
      return id;
    }

    public String getRange() {
      return range;
    }
  }

  public static class DependencyGroup
  {
    @Nullable
    private final String targetFramework;

    private final List<Dependency> dependencies;

    public DependencyGroup(@Nullable final String targetFramework, final List<Dependency> dependencies) {
      this.targetFramework = emptyToNull(targetFramework);
      this.dependencies = dependencies;
    }

    @Nullable
    public String getTargetFramework() {
      return targetFramework;
    }

    public List<Dependency> getDependencies() {
      return dependencies;
    }
  }

  public String getId() {
    return id;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public String getLowerId() {
    return id.toLowerCase(Locale.ROOT);
  }

  public NugetVersion getVersion() {
    return version;
  }

  public void setVersion(final NugetVersion version) {
    this.version = version;
  }

  public String text(final String key) {
    return nullToEmpty(text.get(key));
  }

  public void setText(final String key, @Nullable final String value) {
    if (value == null || value.isBlank()) {
      text.remove(key);
    }
    else {
      text.put(key, value.trim());
    }
  }

  public List<String> getTagList() {
    String tags = text(TAGS).trim();
    if (tags.isEmpty()) {
      return Collections.emptyList();
    }
    return Arrays.stream(tags.split("[\\s,;]+")).filter(tag -> !tag.isEmpty()).toList();
  }

  public List<String> getAuthorList() {
    return splitList(text("authors"));
  }

  public List<String> getOwnerList() {
    return splitList(text("owners"));
  }

  private static List<String> splitList(final String value) {
    if (value.isBlank()) {
      return Collections.emptyList();
    }
    return Arrays.stream(value.split(",")).map(String::trim).filter(item -> !item.isEmpty()).toList();
  }

  public boolean isRequireLicenseAcceptance() {
    return requireLicenseAcceptance;
  }

  public void setRequireLicenseAcceptance(final boolean requireLicenseAcceptance) {
    this.requireLicenseAcceptance = requireLicenseAcceptance;
  }

  public boolean isDevelopmentDependency() {
    return developmentDependency;
  }

  public void setDevelopmentDependency(final boolean developmentDependency) {
    this.developmentDependency = developmentDependency;
  }

  public boolean isListed() {
    return listed;
  }

  public void setListed(final boolean listed) {
    this.listed = listed;
  }

  public long getPackageSize() {
    return packageSize;
  }

  public void setPackageSize(final long packageSize) {
    this.packageSize = packageSize;
  }

  public List<DependencyGroup> getDependencyGroups() {
    return dependencyGroups;
  }

  public void setDependencyGroups(final List<DependencyGroup> dependencyGroups) {
    this.dependencyGroups = dependencyGroups;
  }

  public boolean isSemVer2() {
    if (version.isSemVer2()) {
      return true;
    }
    for (DependencyGroup group : dependencyGroups) {
      for (Dependency dependency : group.getDependencies()) {
        if (rangeIsSemVer2(dependency.getRange())) {
          return true;
        }
      }
    }
    return false;
  }

  private static boolean rangeIsSemVer2(final String range) {
    for (String part : range.replaceAll("[\\[\\]()\\s]", "").split(",")) {
      if (!part.isEmpty() && NugetVersion.parse(part).map(NugetVersion::isSemVer2).orElse(false)) {
        return true;
      }
    }
    return false;
  }

  public Map<String, Object> toAttributes() {
    Map<String, Object> attributes = new LinkedHashMap<>();
    attributes.put(ID, id);
    attributes.put(VERSION, version.toFullString());
    attributes.put(NORMALIZED_VERSION, version.toNormalizedString());
    attributes.putAll(text);
    attributes.put("require_license_acceptance", requireLicenseAcceptance);
    attributes.put("development_dependency", developmentDependency);
    attributes.put("listed", listed);
    attributes.put("package_size", packageSize);
    attributes.put("is_prerelease", version.isPrerelease());
    attributes.put("is_semver2", isSemVer2());
    List<Map<String, Object>> groups = new ArrayList<>();
    for (DependencyGroup group : dependencyGroups) {
      Map<String, Object> groupMap = new LinkedHashMap<>();
      if (group.getTargetFramework() != null) {
        groupMap.put("target_framework", group.getTargetFramework());
      }
      List<Map<String, Object>> dependencies = new ArrayList<>();
      for (Dependency dependency : group.getDependencies()) {
        Map<String, Object> dependencyMap = new LinkedHashMap<>();
        dependencyMap.put(ID, dependency.getId());
        dependencyMap.put("range", dependency.getRange());
        dependencies.add(dependencyMap);
      }
      groupMap.put("dependencies", dependencies);
      groups.add(groupMap);
    }
    attributes.put("dependency_groups", groups);
    return attributes;
  }

  @Nullable
  @SuppressWarnings("unchecked")
  public static NugetPackageMetadata fromAttributes(@Nullable final Map<String, Object> attributes) {
    if (attributes == null || !(attributes.get(ID) instanceof String id)) {
      return null;
    }
    NugetVersion version = NugetVersion.parse((String) attributes.get(VERSION)).orElse(null);
    if (version == null) {
      return null;
    }
    NugetPackageMetadata metadata = new NugetPackageMetadata();
    metadata.setId(id);
    metadata.setVersion(version);
    for (String field : TEXT_FIELDS) {
      Object value = attributes.get(field);
      if (value instanceof String string) {
        metadata.setText(field, string);
      }
    }
    metadata.setRequireLicenseAcceptance(Boolean.TRUE.equals(attributes.get("require_license_acceptance")));
    metadata.setDevelopmentDependency(Boolean.TRUE.equals(attributes.get("development_dependency")));
    metadata.setListed(!Boolean.FALSE.equals(attributes.get("listed")));
    if (attributes.get("package_size") instanceof Number size) {
      metadata.setPackageSize(size.longValue());
    }
    List<DependencyGroup> groups = new ArrayList<>();
    if (attributes.get("dependency_groups") instanceof List<?> groupList) {
      for (Object groupObject : groupList) {
        if (!(groupObject instanceof Map<?, ?> groupMap)) {
          continue;
        }
        List<Dependency> dependencies = new ArrayList<>();
        if (groupMap.get("dependencies") instanceof List<?> dependencyList) {
          for (Object dependencyObject : dependencyList) {
            if (dependencyObject instanceof Map<?, ?> dependencyMap && dependencyMap.get(ID) instanceof String depId) {
              Object range = dependencyMap.get("range");
              dependencies.add(new Dependency(depId, range instanceof String rangeString ? rangeString : ""));
            }
          }
        }
        Object framework = groupMap.get("target_framework");
        groups.add(new DependencyGroup(framework instanceof String frameworkString ? frameworkString : null,
            dependencies));
      }
    }
    metadata.setDependencyGroups(groups);
    return metadata;
  }
}
