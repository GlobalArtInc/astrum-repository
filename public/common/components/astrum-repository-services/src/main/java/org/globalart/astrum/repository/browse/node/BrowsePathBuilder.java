
package org.globalart.astrum.repository.browse.node;

import java.util.ArrayList;
import java.util.List;

import static org.apache.commons.lang3.StringUtils.appendIfMissing;
import static org.apache.commons.lang3.StringUtils.removeStart;
import static org.globalart.astrum.repository.browse.node.BrowsePath.SLASH;
import static org.globalart.astrum.repository.browse.node.BrowsePath.SLASH_CHAR;

/**
 * Static methods for building {@link BrowsePath}.
 *
 * @since 3.25
 */
public class BrowsePathBuilder
{
  private BrowsePathBuilder() {
    // static utility class
  }

  public static List<BrowsePath> fromPaths(final List<String> paths, final boolean trailingSlash) {
    List<BrowsePath> results = new ArrayList<>();

    StringBuilder requestPath = new StringBuilder().append(SLASH_CHAR);
    for (int i = 0; i < paths.size(); i++) {
      requestPath.append(paths.get(i));
      if (trailingSlash || i < paths.size() - 1) {
        requestPath.append(SLASH_CHAR);
      }
      results.add(new BrowsePath(paths.get(i), requestPath.toString()));
    }

    return results;
  }

  public static void appendPath(final List<BrowsePath> browsePaths, final String path) {
    appendPath(browsePaths, path, combine(getLastRequestPath(browsePaths), path));
  }

  private static String combine(final String browsePath, final String path) {
    return appendIfMissing(browsePath, SLASH) + removeStart(path, SLASH);
  }

  private static String getLastRequestPath(final List<BrowsePath> browsePaths) {
    return browsePaths.isEmpty() ? "" : last(browsePaths).getRequestPath();
  }

  public static <T> T last(List<T> items) {
    return items.get(items.size() - 1);
  }

  public static void appendPath(final List<BrowsePath> browsePaths, final String path, final String requestPath) {
    browsePaths.add(new BrowsePath(path, requestPath));
  }
}
