
package org.globalart.astrum.repository.importtask;

/**
 * Utility methods for working with asset paths during import operations.
 */
public final class AssetPathUtils
{
  private AssetPathUtils() {
    // utility class
  }

  /**
   * Extracts the filename from an asset path.
   * <p>
   * This method extracts the last path component from an asset path by finding the last
   * forward slash and returning everything after it. If no slash is found, returns the
   * entire path as the filename.
   *
   * @param assetPath the full asset path (e.g., "path/to/file.gem" or "file.rpm")
   * @return the extracted filename (e.g., "file.gem" or "file.rpm")
   */
  public static String extractFilename(final String assetPath) {
    if (assetPath == null) {
      return null;
    }

    int lastSlash = assetPath.lastIndexOf('/');
    if (lastSlash >= 0) {
      return assetPath.substring(lastSlash + 1);
    }

    return assetPath;
  }
}
