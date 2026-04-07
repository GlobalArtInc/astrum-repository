package org.globalart.astrum.repository.docker.internal.v2;

import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.TreeSet;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Parameters;

public final class DockerPagination
{
  private DockerPagination() {
  }

  public static class Page
  {
    private final List<String> items;

    private final String nextLink;

    Page(final List<String> items, @Nullable final String nextLink) {
      this.items = items;
      this.nextLink = nextLink;
    }

    public List<String> getItems() {
      return items;
    }

    @Nullable
    public String getNextLink() {
      return nextLink;
    }
  }

  public static Page page(final Collection<String> values, final Parameters parameters, final String baseUrl) {
    TreeSet<String> sorted = new TreeSet<>(values);
    String last = parameters.get("last");
    Integer limit = parseLimit(parameters.get("n"));
    Collection<String> remaining = last == null || last.isEmpty() ? sorted : sorted.tailSet(last, false);
    List<String> items = new ArrayList<>();
    for (String value : remaining) {
      if (limit != null && items.size() >= limit) {
        break;
      }
      items.add(value);
    }
    String next = null;
    if (limit != null && !items.isEmpty() && remaining.size() > items.size()) {
      String lastItem = items.get(items.size() - 1);
      next = "<" + baseUrl + "?n=" + limit + "&last=" + URLEncoder.encode(lastItem, StandardCharsets.UTF_8)
          + ">; rel=\"next\"";
    }
    return new Page(items, next);
  }

  @Nullable
  private static Integer parseLimit(@Nullable final String value) {
    if (value == null || value.isBlank()) {
      return null;
    }
    try {
      int limit = Integer.parseInt(value.trim());
      return limit < 0 ? null : limit;
    }
    catch (NumberFormatException e) {
      return null;
    }
  }
}
