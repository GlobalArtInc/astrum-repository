
package org.globalart.astrum.repository.apt.internal;

/**
 * @since 3.17
 */
public class ReleaseName
{
  private ReleaseName() {
    throw new IllegalAccessError("Utility class");
  }

  public static final String RELEASE = "Release";

  public static final String RELEASE_GPG = "Release.gpg";

  public static final String INRELEASE = "InRelease";
}
