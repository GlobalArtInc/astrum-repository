package org.globalart.astrum.repository.swift.internal;

import java.util.Collection;
import java.util.LinkedHashSet;
import java.util.Locale;
import java.util.Set;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

public final class PackageUrls
{
  private static final Pattern SCHEME = Pattern.compile("^[a-z][a-z0-9+.-]*://");

  private static final Pattern SCP_LIKE = Pattern.compile("^[^/@]+@[^/:]+:.+");

  private static final int MAX_LENGTH = 2048;

  private PackageUrls() {
  }

  @Nullable
  public static String normalize(@Nullable final String url) {
    if (url == null) {
      return null;
    }
    String value = url.strip().toLowerCase(Locale.ROOT);
    if (value.isEmpty() || value.length() > MAX_LENGTH) {
      return null;
    }
    boolean scpLike = !SCHEME.matcher(value).find() && SCP_LIKE.matcher(value).matches();
    value = SCHEME.matcher(value).replaceFirst("");
    int query = indexOfAny(value, '?', '#');
    if (query >= 0) {
      value = value.substring(0, query);
    }
    int slash = value.indexOf('/');
    String authority = slash < 0 ? value : value.substring(0, slash);
    String path = slash < 0 ? "" : value.substring(slash);
    int at = authority.lastIndexOf('@');
    if (at >= 0) {
      authority = authority.substring(at + 1);
    }
    if (scpLike) {
      int colon = authority.indexOf(':');
      if (colon >= 0) {
        path = "/" + authority.substring(colon + 1) + path;
        authority = authority.substring(0, colon);
      }
    }
    else if (authority.endsWith(":443") || authority.endsWith(":80") || authority.endsWith(":22")) {
      authority = authority.substring(0, authority.lastIndexOf(':'));
    }
    while (path.endsWith("/")) {
      path = path.substring(0, path.length() - 1);
    }
    if (path.endsWith(".git")) {
      path = path.substring(0, path.length() - 4);
    }
    while (path.contains("//")) {
      path = path.replace("//", "/");
    }
    String normalized = authority + path;
    return normalized.isEmpty() || authority.isEmpty() ? null : normalized;
  }

  public static Set<String> normalizeAll(final Collection<String> urls) {
    Set<String> normalized = new LinkedHashSet<>();
    for (String url : urls) {
      String value = normalize(url);
      if (value != null) {
        normalized.add(value);
      }
    }
    return normalized;
  }

  private static int indexOfAny(final String value, final char... characters) {
    int index = -1;
    for (char character : characters) {
      int found = value.indexOf(character);
      if (found >= 0 && (index < 0 || found < index)) {
        index = found;
      }
    }
    return index;
  }
}
