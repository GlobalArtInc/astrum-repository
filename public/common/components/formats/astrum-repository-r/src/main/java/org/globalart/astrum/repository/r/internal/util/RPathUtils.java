
package org.globalart.astrum.repository.r.internal.util;

import java.util.List;
import java.util.Locale;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.globalart.astrum.repository.view.Context;

import org.apache.commons.lang3.StringUtils;

public final class RPathUtils
{
  public static final String PACKAGES = "PACKAGES";

  public static final String PACKAGES_GZ = "PACKAGES.gz";

  public static final String PACKAGES_RDS = "PACKAGES.rds";

  public static final List<String> ARCHIVE_EXTENSIONS = List.of(".tar.gz", ".tgz", ".zip");

  private static final Pattern PATTERN_PATH = Pattern.compile(".+/.+/.+");

  private static final Pattern ARCHIVE_FILE_NAME = Pattern.compile("([A-Za-z0-9.]+)_([^_/]+)(\\.tar\\.gz|\\.tgz|\\.zip)");

  public static String requestPath(final Context context) {
    return normalize(context.getRequest().getPath());
  }

  public static String normalize(final String path) {
    return StringUtils.prependIfMissing(StringUtils.stripStart(path, "/"), "/");
  }

  public static String buildPath(final String directory, final String fileName) {
    return normalize(StringUtils.appendIfMissing(StringUtils.strip(directory, "/"), "/") + fileName);
  }

  public static String directory(final String path) {
    String normalized = normalize(path);
    return normalized.substring(0, normalized.lastIndexOf('/'));
  }

  public static String namespace(final String path) {
    return StringUtils.stripStart(directory(path), "/");
  }

  public static String fileName(final String path) {
    return path.substring(path.lastIndexOf('/') + 1);
  }

  public static boolean isArchive(final String path) {
    String lowerCase = path.toLowerCase(Locale.ENGLISH);
    return ARCHIVE_EXTENSIONS.stream().anyMatch(extension -> lowerCase.endsWith(extension)
        && lowerCase.length() > extension.length() && lowerCase.charAt(lowerCase.length() - extension.length() - 1) != '/');
  }

  public static boolean isValidRepoPath(final String path) {
    return PATTERN_PATH.matcher(StringUtils.stripStart(path, "/")).matches();
  }

  public static String archiveExtension(final String path) {
    String lowerCase = path.toLowerCase(Locale.ENGLISH);
    return ARCHIVE_EXTENSIONS.stream()
        .filter(lowerCase::endsWith)
        .findFirst()
        .map(extension -> path.substring(path.length() - extension.length()))
        .orElseThrow(() -> new IllegalArgumentException("Not an R package archive: " + path));
  }

  public static String archiveFileName(final String name, final String version, final String extension) {
    return name + "_" + version + extension;
  }

  public static Optional<String[]> nameAndVersion(final String fileName) {
    Matcher matcher = ARCHIVE_FILE_NAME.matcher(fileName);
    if (!matcher.matches()) {
      return Optional.empty();
    }
    return Optional.of(new String[]{matcher.group(1), matcher.group(2)});
  }

  public static String packagesPath(final String directory, final boolean gzip) {
    return buildPath(directory, gzip ? PACKAGES_GZ : PACKAGES);
  }

  public static boolean isGzip(final String path) {
    return path.endsWith(".gz");
  }

  private RPathUtils() {
  }
}
