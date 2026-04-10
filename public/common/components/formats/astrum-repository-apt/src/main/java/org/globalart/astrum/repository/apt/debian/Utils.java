
package org.globalart.astrum.repository.apt.debian;

/**
 * @since 3.17
 */
public class Utils
{
  private Utils() {
  }

  public static boolean isDebPackageContentType(final String path) {
    return path.endsWith(".deb") || path.endsWith(".udeb");
  }
}
