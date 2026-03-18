
package org.globalart.astrum.repository.raw;

/**
 * Helper methods for extracting component/asset coordinates for raw artifacts.
 *
 * @since 3.0
 */
public class RawCoordinatesHelper
{
  public static String getGroup(String path) {
    StringBuilder group = new StringBuilder();
    int i = path.lastIndexOf('/');
    if (!path.startsWith("/") || i == 0) {
      group.append("/");
    }
    if (i != -1) {
      group.append(path, 0, i);
    }
    return group.toString();
  }

  private RawCoordinatesHelper() {
    // Don't instantiate
  }
}
