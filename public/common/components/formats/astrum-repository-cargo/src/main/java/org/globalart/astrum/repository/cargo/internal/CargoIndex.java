/*
 * Copyright 2019, Imperva, Inc. All rights reserved.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Eclipse Public License Version 1.0, which accompanies this
 * distribution and is available at http://www.eclipse.org/legal/epl-v10.html.
 *
 * Imperva, the Imperva logo, SecureSphere, Incapsula, CounterBreach,
 * ThreatRadar, Camouflage, Attack Analytics, Prevoty and design are trademarks
 * of Imperva, Inc. and its subsidiaries. All other brand or product names are
 * trademarks or registered trademarks of their respective owners.
 */
package org.globalart.astrum.repository.cargo.internal;

import java.io.ByteArrayOutputStream;
import java.time.Instant;
import java.time.temporal.ChronoUnit;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;

import javax.annotation.Nullable;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;

import static java.nio.charset.StandardCharsets.UTF_8;

public final class CargoIndex
{
  private CargoIndex() {
  }

  public static ObjectNode entry(final JsonNode metadata, final String cksum, final Instant published) {
    ObjectNode entry = CargoResponses.object();
    entry.put("name", text(metadata, "name"));
    entry.put("vers", text(metadata, "vers"));
    ArrayNode deps = entry.putArray("deps");
    JsonNode sourceDeps = metadata.get("deps");
    if (sourceDeps != null && sourceDeps.isArray()) {
      for (JsonNode dep : sourceDeps) {
        deps.add(dependency(dep));
      }
    }
    entry.put("cksum", cksum);
    ObjectNode features = entry.putObject("features");
    ObjectNode features2 = CargoResponses.object();
    JsonNode sourceFeatures = metadata.get("features");
    if (sourceFeatures != null && sourceFeatures.isObject()) {
      Iterator<Entry<String, JsonNode>> fields = sourceFeatures.fields();
      while (fields.hasNext()) {
        Entry<String, JsonNode> feature = fields.next();
        ArrayNode values = stringArray(feature.getValue());
        if (usesNewFeatureSyntax(values)) {
          features2.set(feature.getKey(), values);
        }
        else {
          features.set(feature.getKey(), values);
        }
      }
    }
    entry.put("yanked", false);
    String links = text(metadata, "links");
    if (links != null) {
      entry.put("links", links);
    }
    if (!features2.isEmpty()) {
      entry.put("v", 2);
      entry.set("features2", features2);
    }
    String rustVersion = text(metadata, "rust_version");
    if (rustVersion != null) {
      entry.put("rust_version", rustVersion);
    }
    entry.put("pubtime", published.truncatedTo(ChronoUnit.SECONDS).toString());
    return entry;
  }

  private static ObjectNode dependency(final JsonNode dep) {
    String name = text(dep, "name");
    String explicitName = text(dep, "explicit_name_in_toml");
    ObjectNode result = CargoResponses.object();
    result.put("name", explicitName != null ? explicitName : name);
    String req = text(dep, "version_req");
    result.put("req", req != null ? req : "*");
    result.set("features", stringArray(dep.get("features")));
    result.put("optional", bool(dep, "optional", false));
    result.put("default_features", bool(dep, "default_features", true));
    String target = text(dep, "target");
    if (target != null) {
      result.put("target", target);
    }
    else {
      result.putNull("target");
    }
    String kind = text(dep, "kind");
    result.put("kind", kind != null ? kind : "normal");
    String registry = text(dep, "registry");
    if (registry != null) {
      result.put("registry", registry);
    }
    if (explicitName != null) {
      result.put("package", name);
    }
    return result;
  }

  private static boolean usesNewFeatureSyntax(final ArrayNode values) {
    for (JsonNode value : values) {
      String text = value.asText();
      if (text.startsWith("dep:") || text.contains("?/")) {
        return true;
      }
    }
    return false;
  }

  private static ArrayNode stringArray(@Nullable final JsonNode node) {
    ArrayNode array = CargoResponses.MAPPER.createArrayNode();
    if (node != null && node.isArray()) {
      for (JsonNode value : node) {
        if (value.isTextual()) {
          array.add(value.asText());
        }
      }
    }
    return array;
  }

  @Nullable
  public static String text(@Nullable final JsonNode node, final String field) {
    if (node == null) {
      return null;
    }
    JsonNode value = node.get(field);
    return value == null || value.isNull() ? null : value.asText();
  }

  private static boolean bool(final JsonNode node, final String field, final boolean defaultValue) {
    JsonNode value = node.get(field);
    return value == null || value.isNull() ? defaultValue : value.asBoolean(defaultValue);
  }

  public static String withYanked(final String entry, final boolean yanked) {
    JsonNode node = CargoResponses.read(entry.getBytes(UTF_8));
    if (node instanceof ObjectNode object) {
      object.put("yanked", yanked);
      return object.toString();
    }
    return entry;
  }

  public static List<String> lines(final byte[] content) {
    List<String> lines = new ArrayList<>();
    for (String line : new String(content, UTF_8).split("\n")) {
      if (!line.isBlank()) {
        lines.add(line.strip());
      }
    }
    return lines;
  }

  @Nullable
  public static String version(final String line) {
    try {
      return text(CargoResponses.MAPPER.readTree(line), "vers");
    }
    catch (Exception e) {
      return null;
    }
  }

  public static byte[] merge(final List<byte[]> files) {
    Map<String, String> entries = new LinkedHashMap<>();
    for (byte[] file : files) {
      for (String line : lines(file)) {
        String version = version(line);
        if (version != null) {
          entries.putIfAbsent(version, line);
        }
      }
    }
    return join(entries.values());
  }

  public static byte[] join(final Iterable<String> lines) {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    for (String line : lines) {
      out.writeBytes(line.getBytes(UTF_8));
      out.write('\n');
    }
    return out.toByteArray();
  }
}
