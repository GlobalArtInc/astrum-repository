
package org.globalart.astrum.repository.cocoapods.internal;

import java.net.URI;
import java.nio.charset.StandardCharsets;

import com.google.common.escape.Escaper;
import com.google.common.hash.Hashing;
import com.google.common.net.UrlEscapers;

public class PathUtils
{
  public static final String SPECS_DIRECTORY = "Specs";

  public static final String PODS_DIRECTORY = "pods";

  private static final String SPECS_PATH_FORMAT = SPECS_DIRECTORY + "/%s/%s/%s/%s.podspec.json";

  private static final String SPECS_SHARDS_PATH_FORMAT = "%s/%s/%s";

  private static final Escaper SEGMENT_ESCAPER = UrlEscapers.urlPathSegmentEscaper();

  private PathUtils() {
  }

  public static String buildSpecFilePath(final String name, final String version) {
    return String.format(SPECS_PATH_FORMAT, getSpecShardPath(name), name, version, name);
  }

  public static String buildPodPath(final String name, final String version, final URI downloadUri) {
    return String.join("/", PODS_DIRECTORY, name, version, podFileName(version, downloadUri));
  }

  public static URI buildPodUri(final URI repositoryUri, final String name, final String version, final URI downloadUri) {
    String relative = String.join("/",
        PODS_DIRECTORY,
        SEGMENT_ESCAPER.escape(name),
        SEGMENT_ESCAPER.escape(version),
        SEGMENT_ESCAPER.escape(podFileName(version, downloadUri)));
    return repositoryUri.resolve(relative);
  }

  public static String podFileName(final String version, final URI downloadUri) {
    String path = downloadUri.getPath();
    String fileName = path == null ? "" : path.substring(path.lastIndexOf('/') + 1);
    return fileName.isEmpty() ? version : fileName;
  }

  public static boolean isValidSegment(final String segment) {
    return !segment.isEmpty() && !".".equals(segment) && !"..".equals(segment) && segment.indexOf('/') < 0;
  }

  @SuppressWarnings("deprecation")
  private static String getSpecShardPath(final String name) {
    String nameHash = Hashing.md5().hashString(name, StandardCharsets.UTF_8).toString();
    return String.format(SPECS_SHARDS_PATH_FORMAT,
        nameHash.substring(0, 1), nameHash.substring(1, 2), nameHash.substring(2, 3));
  }
}
