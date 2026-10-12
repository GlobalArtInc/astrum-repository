package org.globalart.astrum.repository.yum.internal;

import java.util.Arrays;
import java.util.List;
import java.util.Optional;

import org.globalart.astrum.repository.view.Context;

public final class YumPaths
{
  public static final String REPODATA = "repodata";

  public static final String REPOMD = "repomd.xml";

  public static final String SIGNATURE = "repomd.xml.asc";

  public static final String PUBLIC_KEY = "repomd.xml.key";

  public static final String RPM_EXTENSION = ".rpm";

  private YumPaths() {
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

  public static List<String> segments(final String path) {
    return Arrays.stream(path.split("/")).filter(segment -> !segment.isEmpty()).toList();
  }

  public static String fileName(final String path) {
    return path.substring(path.lastIndexOf('/') + 1);
  }

  public static String directory(final String path) {
    int slash = path.lastIndexOf('/');
    return slash <= 0 ? "" : path.substring(0, slash);
  }

  public static boolean isRpm(final String path) {
    return path.endsWith(RPM_EXTENSION);
  }

  public static boolean isInRepodata(final String path) {
    return segments(path).contains(REPODATA);
  }

  public static Optional<String> base(final String packagePath, final int depth) {
    List<String> segments = segments(packagePath);
    if (segments.size() - 1 < depth) {
      return Optional.empty();
    }
    return Optional.of(depth == 0 ? "" : "/" + String.join("/", segments.subList(0, depth)));
  }

  public static Optional<String> baseOfMetadata(final String path) {
    List<String> segments = segments(path);
    if (segments.size() < 2 || !REPODATA.equals(segments.get(segments.size() - 2))) {
      return Optional.empty();
    }
    List<String> base = segments.subList(0, segments.size() - 2);
    return Optional.of(base.isEmpty() ? "" : "/" + String.join("/", base));
  }

  public static int depthOf(final String base) {
    return segments(base).size();
  }

  public static String repodataDirectory(final String base) {
    return base + "/" + REPODATA;
  }

  public static String repomdPath(final String base) {
    return repodataDirectory(base) + "/" + REPOMD;
  }

  public static String signaturePath(final String base) {
    return repodataDirectory(base) + "/" + SIGNATURE;
  }

  public static String location(final String base, final String packagePath) {
    return packagePath.substring(base.length() + 1);
  }
}
