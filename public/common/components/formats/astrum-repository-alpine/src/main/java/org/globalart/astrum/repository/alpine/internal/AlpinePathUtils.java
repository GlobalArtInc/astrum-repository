
package org.globalart.astrum.repository.alpine.internal;

import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.globalart.astrum.repository.view.Context;

public final class AlpinePathUtils
{
  public static final String APKINDEX = "APKINDEX.tar.gz";

  public static final String PACKAGE_EXTENSION = ".apk";

  private static final Pattern PACKAGE_FILE_NAME = Pattern.compile("^(.+?)-([0-9][^-/]*(?:-r[0-9]+)?)\\.apk$");

  private AlpinePathUtils() {
  }

  public static String requestPath(final Context context) {
    return normalize(context.getRequest().getPath());
  }

  public static String normalize(final String path) {
    String result = path.replace('\\', '/').replaceAll("/+", "/");
    if (!result.startsWith("/")) {
      result = "/" + result;
    }
    if (result.length() > 1 && result.endsWith("/")) {
      result = result.substring(0, result.length() - 1);
    }
    for (String segment : result.split("/")) {
      if (".".equals(segment) || "..".equals(segment)) {
        throw new IllegalArgumentException("Path must not contain '.' or '..' segments: " + path);
      }
    }
    return result;
  }

  public static String directory(final String path) {
    int slash = path.lastIndexOf('/');
    return slash <= 0 ? "" : path.substring(0, slash);
  }

  public static String fileName(final String path) {
    return path.substring(path.lastIndexOf('/') + 1);
  }

  public static String lastSegment(final String directory) {
    return fileName(directory);
  }

  public static String indexPath(final String directory) {
    return directory + "/" + APKINDEX;
  }

  public static boolean isPackage(final String path) {
    return path.endsWith(PACKAGE_EXTENSION);
  }

  public static Optional<NameAndVersion> nameAndVersion(final String fileName) {
    Matcher matcher = PACKAGE_FILE_NAME.matcher(fileName);
    if (matcher.matches()) {
      return Optional.of(new NameAndVersion(matcher.group(1), matcher.group(2)));
    }
    return Optional.empty();
  }

  public record NameAndVersion(String name, String version)
  {
  }
}
