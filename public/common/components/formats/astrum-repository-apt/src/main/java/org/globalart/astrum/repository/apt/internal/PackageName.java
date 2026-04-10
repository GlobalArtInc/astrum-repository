
package org.globalart.astrum.repository.apt.internal;

/**
 * @since 3.17
 */
public class PackageName
{
  private PackageName() {
    throw new IllegalAccessError("Utility class");
  }

  public static final String PACKAGES = "Packages";

  public static final String PACKAGES_GZ = "Packages.gz";

  public static final String PACKAGES_BZ2 = "Packages.bz2";

  public static final String PACKAGES_XZ = "Packages.xz";
}
