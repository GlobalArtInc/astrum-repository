package org.globalart.astrum.repository.conan.internal;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import static com.google.common.base.Preconditions.checkNotNull;

public final class ConanInfo
{
  private final String content;

  private final Map<String, List<String>> sections;

  private ConanInfo(final String content, final Map<String, List<String>> sections) {
    this.content = content;
    this.sections = sections;
  }

  public static ConanInfo parse(final String content) {
    checkNotNull(content);
    Map<String, List<String>> sections = new LinkedHashMap<>();
    List<String> current = null;
    for (String raw : content.split("\\r?\\n")) {
      String line = raw.trim();
      if (line.isEmpty() || line.startsWith("#")) {
        continue;
      }
      if (line.startsWith("[") && line.endsWith("]")) {
        current = sections.computeIfAbsent(line.substring(1, line.length() - 1).trim(), key -> new ArrayList<>());
      }
      else if (current != null) {
        current.add(line);
      }
    }
    return new ConanInfo(content, sections);
  }

  public String content() {
    return content;
  }

  public Map<String, String> values(final String section) {
    Map<String, String> values = new LinkedHashMap<>();
    for (String line : sections.getOrDefault(section, List.of())) {
      int eq = line.indexOf('=');
      if (eq > 0) {
        values.put(line.substring(0, eq).trim(), line.substring(eq + 1).trim());
      }
    }
    return values;
  }

  public List<String> lines(final String section) {
    return List.copyOf(sections.getOrDefault(section, List.of()));
  }

  public Map<String, Object> toV2Json() {
    Map<String, Object> json = new LinkedHashMap<>();
    json.put("settings", values("settings"));
    json.put("options", values("options"));
    json.put("requires", lines("requires"));
    json.put("content", content);
    return json;
  }

  public Map<String, Object> toV1Json() {
    Map<String, Object> json = new LinkedHashMap<>();
    json.put("settings", values("settings"));
    json.put("options", values("options"));
    json.put("full_requires", lines("full_requires"));
    List<String> recipeHash = lines("recipe_hash");
    if (!recipeHash.isEmpty()) {
      json.put("recipe_hash", recipeHash.get(0));
    }
    return json;
  }
}
