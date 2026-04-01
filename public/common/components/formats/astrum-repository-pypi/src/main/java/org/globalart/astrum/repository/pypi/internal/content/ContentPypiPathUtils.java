
package org.globalart.astrum.repository.pypi.internal.content;

/**
 * @since 3.29
 */
public class ContentPypiPathUtils
{
  private static final String INDEX_PATH_PREFIX = "/simple/";

  private static final String PACKAGE_PATH_PREFIX = "/packages/";

  private ContentPypiPathUtils() {}

  /**
   * Builds a path to a package for a particular path.
   */
  public static String packagesPath(final String... parts) {
    String pkg = String.join("/", parts);

    return pkg.startsWith(PACKAGE_PATH_PREFIX) ? pkg : PACKAGE_PATH_PREFIX + pkg;
  }

  /**
   * Builds a path to the simple index.
   */
  public static String indexPath() {
    return INDEX_PATH_PREFIX;
  }

  /**
   * Builds a path to the simple index for a particular name.
   */
  public static String indexPath(final String name) {
    return INDEX_PATH_PREFIX + name + (name.endsWith("/") ? "" : "/");
  }

  /**
   * Builds a URI to the simple index.
   */
  public static String indexUriPath() {
    return INDEX_PATH_PREFIX.substring(1);
  }

  /**
   * Builds a URI to the simple index for a particular name.
   */
  public static String indexUriPath(final String name) {
    return buildIndexPath(name).substring(1);
  }

  private static String buildIndexPath(final String name) {
    return INDEX_PATH_PREFIX + name + (name.endsWith("/") ? "" : "/");
  }
}
