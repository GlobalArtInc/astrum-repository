package org.globalart.astrum.repository.swift.internal;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;

import javax.annotation.Nullable;

public final class Links
{
  public record Link(String url, Map<String, String> params)
  {
    public String rel() {
      return params.getOrDefault("rel", "");
    }

    public String param(final String name) {
      return params.get(name);
    }
  }

  private final List<String> values = new ArrayList<>();

  public Links add(final String url, final String rel) {
    if (isLinkable(url)) {
      values.add("<" + url + ">; rel=\"" + rel + "\"");
    }
    return this;
  }

  public Links addManifest(final String url, final String filename, final String toolsVersion) {
    values.add("<" + url + ">; rel=\"alternate\"; filename=\"" + filename + "\"; swift-tools-version=\""
        + toolsVersion + "\"");
    return this;
  }

  public boolean isEmpty() {
    return values.isEmpty();
  }

  @Override
  public String toString() {
    return String.join(", ", values);
  }

  public static boolean isLinkable(@Nullable final String url) {
    return url != null && !url.isBlank() && url.chars().noneMatch(c -> c == '<' || c == '>' || c == ','
        || c == ';' || c == '"' || Character.isWhitespace(c));
  }

  public static List<Link> parse(@Nullable final Iterable<String> headers) {
    List<Link> links = new ArrayList<>();
    if (headers == null) {
      return links;
    }
    for (String header : headers) {
      if (header != null) {
        parseHeader(header, links);
      }
    }
    return links;
  }

  private static void parseHeader(final String header, final List<Link> links) {
    int i = 0;
    while (i < header.length()) {
      int open = header.indexOf('<', i);
      if (open < 0) {
        return;
      }
      int close = header.indexOf('>', open);
      if (close < 0) {
        return;
      }
      String url = header.substring(open + 1, close).trim();
      int end = nextLink(header, close + 1);
      Map<String, String> params = new LinkedHashMap<>();
      for (String param : header.substring(close + 1, end).split(";")) {
        int equals = param.indexOf('=');
        if (equals > 0) {
          String name = param.substring(0, equals).trim().toLowerCase(Locale.ROOT);
          String value = param.substring(equals + 1).trim();
          if (value.length() >= 2 && value.startsWith("\"") && value.endsWith("\"")) {
            value = value.substring(1, value.length() - 1);
          }
          params.put(name, value);
        }
      }
      links.add(new Link(url, params));
      i = end;
    }
  }

  private static int nextLink(final String header, final int from) {
    boolean quoted = false;
    for (int i = from; i < header.length(); i++) {
      char c = header.charAt(i);
      if (c == '"') {
        quoted = !quoted;
      }
      else if (c == ',' && !quoted) {
        return i;
      }
    }
    return header.length();
  }
}
