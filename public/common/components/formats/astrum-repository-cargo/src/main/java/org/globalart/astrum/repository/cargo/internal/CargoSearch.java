package org.globalart.astrum.repository.cargo.internal;

import java.util.LinkedHashMap;
import java.util.Map;

import org.globalart.astrum.repository.view.Context;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;

public final class CargoSearch
{
  static final int DEFAULT_PER_PAGE = 10;

  static final int MAX_PER_PAGE = 100;

  private CargoSearch() {
  }

  public static String query(final Context context) {
    String query = context.getRequest().getParameters().get("q");
    return query == null ? "" : query;
  }

  public static int perPage(final Context context) {
    String value = context.getRequest().getParameters().get("per_page");
    if (value == null) {
      return DEFAULT_PER_PAGE;
    }
    try {
      return Math.max(1, Math.min(MAX_PER_PAGE, Integer.parseInt(value.strip())));
    }
    catch (NumberFormatException e) {
      return DEFAULT_PER_PAGE;
    }
  }

  public static ObjectNode merge(final Iterable<JsonNode> results, final int perPage) {
    Map<String, JsonNode> crates = new LinkedHashMap<>();
    long total = 0;
    for (JsonNode result : results) {
      JsonNode found = result.get("crates");
      if (found != null && found.isArray()) {
        for (JsonNode crate : found) {
          String name = CargoIndex.text(crate, "name");
          if (name != null && crates.putIfAbsent(name, crate) == null) {
            total++;
          }
        }
      }
      JsonNode meta = result.get("meta");
      if (meta != null && meta.has("total") && found != null) {
        total += Math.max(0, meta.get("total").asLong() - found.size());
      }
    }
    ObjectNode merged = CargoResponses.object();
    ArrayNode array = merged.putArray("crates");
    crates.values().stream().limit(perPage).forEach(array::add);
    merged.putObject("meta").put("total", total);
    return merged;
  }
}
