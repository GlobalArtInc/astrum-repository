
package org.globalart.astrum.repository.apt.internal;

/**
 * Apt properties
 *
 * @since 3.31
 */
public final class AptProperties
{
  private AptProperties() {
    // Properties class
  }

  // Apt general properties
  public static final String DEB = "DEB";

  // Apt hosted properties for metadata rebuild
  public static final String P_INDEX_SECTION = "index_section";

  public static final String P_ARCHITECTURE = "architecture";

  public static final String P_PACKAGE_NAME = "package_name";

  public static final String P_PACKAGE_VERSION = "package_version";

  // Apt supported metadata archive file extensions
  public static final String GZ = ".gz";

  public static final String BZ2 = ".bz2";
}
