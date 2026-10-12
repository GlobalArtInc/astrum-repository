package org.globalart.astrum.repository.terraform.internal.proxy;

import java.net.URI;
import java.net.URISyntaxException;
import java.net.URLDecoder;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.terraform.ModuleArchives;

import static java.nio.charset.StandardCharsets.UTF_8;

/**
 * The package location a module registry hands out in {@code X-Terraform-Get}, in the source address syntax of
 * Terraform's getter: an optional forced getter ({@code git::}), the address, an optional subdirectory after
 * {@code //}, and query arguments such as {@code archive} or {@code ref}.
 *
 * A location can be cached when it names an archive over HTTP, or a tag or commit of a GitHub repository, which
 * GitHub serves as a tarball whose single top directory the {@code *} subdirectory selects. Other locations (other
 * Git hosts, SSH, cloud buckets) are handed to Terraform unchanged.
 */
public record ModuleSource(String location, @Nullable String archiveUrl, @Nullable String extension,
                           @Nullable String subdir)
{
  private static final Pattern FORCED = Pattern.compile("([a-z0-9]+)::(.+)");

  private static final Pattern GITHUB =
      Pattern.compile("https://(?:www\\.)?github\\.com/([A-Za-z0-9_.-]+)/([A-Za-z0-9_.-]+?)(?:\\.git)?/?");

  private static final String CODELOAD = "https://codeload.github.com/";

  public boolean cacheable() {
    return archiveUrl != null && extension != null;
  }

  /**
   * The subdirectory suffix to append to a location that points at the cached archive.
   */
  public String subdirSuffix() {
    return subdir == null || subdir.isEmpty() ? "" : "//" + subdir;
  }

  public static ModuleSource parse(final String location) {
    String source = location.strip();
    String forced = null;
    Matcher forcedMatcher = FORCED.matcher(source);
    if (forcedMatcher.matches()) {
      forced = forcedMatcher.group(1);
      source = forcedMatcher.group(2);
    }
    if (source.startsWith("github.com/")) {
      forced = forced == null ? "git" : forced;
      source = "https://" + source;
    }
    String subdir = null;
    int schemeEnd = source.indexOf("://");
    int subdirStart = source.indexOf("//", schemeEnd < 0 ? 0 : schemeEnd + 3);
    if (subdirStart >= 0) {
      String rest = source.substring(subdirStart + 2);
      source = source.substring(0, subdirStart);
      int query = rest.indexOf('?');
      if (query >= 0) {
        source = source + rest.substring(query);
        rest = rest.substring(0, query);
      }
      subdir = rest.isEmpty() ? null : rest;
    }
    URI uri;
    try {
      uri = new URI(source);
    }
    catch (URISyntaxException e) {
      return new ModuleSource(location, null, null, null);
    }
    String scheme = uri.getScheme() == null ? "" : uri.getScheme().toLowerCase(Locale.ROOT);
    if (!"http".equals(scheme) && !"https".equals(scheme)) {
      return new ModuleSource(location, null, null, null);
    }
    if (forced == null || "http".equals(forced) || "https".equals(forced)) {
      return http(location, uri, subdir);
    }
    if ("git".equals(forced)) {
      return github(location, uri, subdir);
    }
    return new ModuleSource(location, null, null, null);
  }

  private static ModuleSource http(final String location, final URI uri, @Nullable final String subdir) {
    List<String[]> query = query(uri.getRawQuery());
    Optional<String> extension = query.stream()
        .filter(pair -> "archive".equals(pair[0]))
        .findFirst()
        .flatMap(pair -> ModuleArchives.fromArchiveParameter(pair[1]));
    if (extension.isEmpty()) {
      extension = ModuleArchives.extension(uri.getPath());
    }
    if (extension.isEmpty()) {
      return new ModuleSource(location, null, null, null);
    }
    StringBuilder url = new StringBuilder(uri.getScheme()).append("://").append(uri.getRawAuthority())
        .append(uri.getRawPath() == null ? "" : uri.getRawPath());
    List<String> kept = query.stream()
        .filter(pair -> !"archive".equals(pair[0]))
        .map(pair -> pair[1] == null ? pair[0] : pair[0] + "=" + pair[1])
        .toList();
    if (!kept.isEmpty()) {
      url.append('?').append(String.join("&", kept));
    }
    return new ModuleSource(location, url.toString(), extension.get(), subdir);
  }

  private static ModuleSource github(final String location, final URI uri, @Nullable final String subdir) {
    String withoutQuery = uri.getScheme().toLowerCase(Locale.ROOT) + "://" + uri.getRawAuthority()
        + (uri.getRawPath() == null ? "" : uri.getRawPath());
    Matcher matcher = GITHUB.matcher(withoutQuery);
    if (!matcher.matches()) {
      return new ModuleSource(location, null, null, null);
    }
    String ref = "HEAD";
    for (String[] pair : query(uri.getRawQuery())) {
      if ("ref".equals(pair[0]) && pair[1] != null && !pair[1].isBlank()) {
        ref = URLDecoder.decode(pair[1], UTF_8);
      }
      else if ("sshkey".equals(pair[0])) {
        return new ModuleSource(location, null, null, null);
      }
    }
    if (!ref.matches("[A-Za-z0-9._/-]+") || ref.contains("..")) {
      return new ModuleSource(location, null, null, null);
    }
    String archive = CODELOAD + matcher.group(1) + "/" + matcher.group(2) + "/tar.gz/" + ref;
    String glob = subdir == null ? "*" : "*/" + subdir;
    return new ModuleSource(location, archive, ModuleArchives.TAR_GZ, glob);
  }

  private static List<String[]> query(@Nullable final String rawQuery) {
    List<String[]> pairs = new ArrayList<>();
    if (rawQuery == null || rawQuery.isEmpty()) {
      return pairs;
    }
    for (String part : rawQuery.split("&")) {
      if (part.isEmpty()) {
        continue;
      }
      int equals = part.indexOf('=');
      pairs.add(equals < 0 ? new String[]{part, null} : new String[]{part.substring(0, equals),
          part.substring(equals + 1)});
    }
    return pairs;
  }

  /**
   * Resolves a location relative to the download endpoint that returned it, as Terraform does for locations that
   * start with {@code /}, {@code ./} or {@code ../}.
   */
  public static String resolve(final URI downloadEndpoint, final String location) {
    String value = location.strip();
    if (!value.startsWith("/") && !value.startsWith("./") && !value.startsWith("../")) {
      return value;
    }
    int subdir = value.indexOf("//", 1);
    String suffix = "";
    if (subdir > 0) {
      suffix = value.substring(subdir);
      value = value.substring(0, subdir);
      int query = suffix.indexOf('?');
      if (query >= 0) {
        value = value + suffix.substring(query);
        suffix = suffix.substring(0, query);
      }
    }
    URI resolved = downloadEndpoint.resolve(value);
    if (suffix.isEmpty()) {
      return resolved.toString();
    }
    String query = resolved.getRawQuery() == null ? "" : "?" + resolved.getRawQuery();
    String base = resolved.toString();
    return base.substring(0, base.length() - query.length()) + suffix + query;
  }
}
