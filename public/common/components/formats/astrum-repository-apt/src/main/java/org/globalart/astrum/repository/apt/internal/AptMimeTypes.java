
package org.globalart.astrum.repository.apt.internal;

/**
 * @since 3.17
 */
public class AptMimeTypes
{
  private AptMimeTypes() {
    throw new IllegalAccessError("Utility class");
  }

  public static final String TEXT = "text/plain";

  public static final String GZIP = "application/gzip";

  public static final String BZIP = "application/bzip2";

  public static final String XZ = "application/x-xz";

  public static final String SIGNATURE = "application/pgp-signature";

  public static final String PUBLICKEY = "application/pgp";

  public static final String PACKAGE = "application/vnd.debian.binary-package";
}
