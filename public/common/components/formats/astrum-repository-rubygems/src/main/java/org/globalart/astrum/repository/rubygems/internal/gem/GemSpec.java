package org.globalart.astrum.repository.rubygems.internal.gem;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;

import javax.annotation.Nullable;

public record GemSpec(
    String name,
    String version,
    String platform,
    @Nullable String summary,
    @Nullable String description,
    List<String> authors,
    List<String> email,
    @Nullable String homepage,
    List<String> licenses,
    Map<String, String> metadata,
    List<GemDependency> dependencies,
    List<String> requiredRubyVersion,
    List<String> requiredRubygemsVersion,
    @Nullable String rubygemsVersion,
    int specificationVersion,
    @Nullable String date)
{
  public static final String NAME = "name";

  public static final String VERSION = "version";

  public static final String PLATFORM = "platform";

  public static final String SUMMARY = "summary";

  public static final String DESCRIPTION = "description";

  public static final String AUTHORS = "authors";

  public static final String EMAIL = "email";

  public static final String HOMEPAGE = "homepage";

  public static final String LICENSES = "licenses";

  public static final String METADATA = "metadata";

  public static final String DEPENDENCIES = "dependencies";

  public static final String REQUIRED_RUBY_VERSION = "required_ruby_version";

  public static final String REQUIRED_RUBYGEMS_VERSION = "required_rubygems_version";

  public static final String RUBYGEMS_VERSION = "rubygems_version";

  public static final String SPECIFICATION_VERSION = "specification_version";

  public static final String DATE = "date";

  private static final String AUTHOR_SEPARATOR = ", ";

  public GemSpec {
    platform = GemPlatform.normalize(platform);
    authors = List.copyOf(authors);
    email = List.copyOf(email);
    licenses = List.copyOf(licenses);
    metadata = Map.copyOf(metadata);
    dependencies = List.copyOf(dependencies);
    requiredRubyVersion = requiredRubyVersion.isEmpty() ? List.of(GemRequirements.ANY)
        : List.copyOf(requiredRubyVersion);
    requiredRubygemsVersion = requiredRubygemsVersion.isEmpty() ? List.of(GemRequirements.ANY)
        : List.copyOf(requiredRubygemsVersion);
  }

  public GemCoordinates coordinates() {
    return GemCoordinates.of(this);
  }

  public List<GemDependency> runtimeDependencies() {
    return dependencies.stream().filter(GemDependency::isRuntime).toList();
  }

  public Map<String, Object> toAttributes() {
    Map<String, Object> attributes = new LinkedHashMap<>();
    attributes.put(NAME, name);
    attributes.put(VERSION, version);
    attributes.put(PLATFORM, platform);
    putIfPresent(attributes, SUMMARY, summary);
    putIfPresent(attributes, DESCRIPTION, description);
    if (!authors.isEmpty()) {
      attributes.put(AUTHORS, String.join(AUTHOR_SEPARATOR, authors));
    }
    if (!email.isEmpty()) {
      attributes.put(EMAIL, email);
    }
    putIfPresent(attributes, HOMEPAGE, homepage);
    if (!licenses.isEmpty()) {
      attributes.put(LICENSES, licenses);
    }
    if (!metadata.isEmpty()) {
      attributes.put(METADATA, new LinkedHashMap<>(metadata));
    }
    List<Map<String, Object>> deps = new ArrayList<>();
    for (GemDependency dependency : dependencies) {
      Map<String, Object> dep = new LinkedHashMap<>();
      dep.put(NAME, dependency.name());
      dep.put("type", dependency.type());
      dep.put("requirements", dependency.requirements());
      deps.add(dep);
    }
    attributes.put(DEPENDENCIES, deps);
    attributes.put(REQUIRED_RUBY_VERSION, requiredRubyVersion);
    attributes.put(REQUIRED_RUBYGEMS_VERSION, requiredRubygemsVersion);
    putIfPresent(attributes, RUBYGEMS_VERSION, rubygemsVersion);
    attributes.put(SPECIFICATION_VERSION, specificationVersion);
    putIfPresent(attributes, DATE, date);
    return attributes;
  }

  @SuppressWarnings("unchecked")
  public static GemSpec fromAttributes(final Map<String, Object> attributes) {
    List<GemDependency> dependencies = new ArrayList<>();
    if (attributes.get(DEPENDENCIES) instanceof List<?> deps) {
      for (Object item : deps) {
        if (item instanceof Map<?, ?> dep && dep.get(NAME) != null) {
          dependencies.add(new GemDependency(String.valueOf(dep.get(NAME)),
              Objects.toString(dep.get("type"), GemDependency.RUNTIME), strings(dep.get("requirements"))));
        }
      }
    }
    Map<String, String> metadata = new LinkedHashMap<>();
    if (attributes.get(METADATA) instanceof Map<?, ?> map) {
      map.forEach((key, value) -> {
        if (key != null && value != null) {
          metadata.put(key.toString(), value.toString());
        }
      });
    }
    Object authors = attributes.get(AUTHORS);
    List<String> authorList = authors instanceof String text ? List.of(text.split(AUTHOR_SEPARATOR)) : strings(authors);
    Object specificationVersion = attributes.get(SPECIFICATION_VERSION);
    return new GemSpec(
        String.valueOf(attributes.get(NAME)),
        String.valueOf(attributes.get(VERSION)),
        (String) attributes.get(PLATFORM),
        (String) attributes.get(SUMMARY),
        (String) attributes.get(DESCRIPTION),
        authorList,
        strings(attributes.get(EMAIL)),
        (String) attributes.get(HOMEPAGE),
        strings(attributes.get(LICENSES)),
        metadata,
        dependencies,
        strings(attributes.get(REQUIRED_RUBY_VERSION)),
        strings(attributes.get(REQUIRED_RUBYGEMS_VERSION)),
        (String) attributes.get(RUBYGEMS_VERSION),
        specificationVersion instanceof Number number ? number.intValue() : 4,
        (String) attributes.get(DATE));
  }

  private static void putIfPresent(final Map<String, Object> attributes, final String key, @Nullable final String value) {
    if (value != null && !value.isBlank()) {
      attributes.put(key, value);
    }
  }

  private static List<String> strings(@Nullable final Object value) {
    List<String> result = new ArrayList<>();
    if (value instanceof List<?> list) {
      for (Object item : list) {
        if (item != null) {
          result.add(item.toString());
        }
      }
    }
    else if (value instanceof String text && !text.isBlank()) {
      result.add(text);
    }
    return result;
  }
}
