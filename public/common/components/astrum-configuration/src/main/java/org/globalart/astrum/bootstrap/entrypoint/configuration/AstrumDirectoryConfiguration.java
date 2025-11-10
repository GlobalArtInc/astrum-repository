
package org.globalart.astrum.bootstrap.entrypoint.configuration;

import java.nio.file.Path;
import java.nio.file.Paths;

/**
 * Do not add logging to this class, it is used early in Astrum startup
 */
public final class AstrumDirectoryConfiguration
{
  private static String compute(
      final String systemPropertyName,
      final String defaultValue)
  {
    String configuredValue = System.getProperty(systemPropertyName, defaultValue);

    // Compute the absolute resolved path without .. or . segments
    String value = Paths.get(configuredValue).toAbsolutePath().normalize().toString();

    // Set the system property so it is available in the rest of Astrum
    System.setProperty(systemPropertyName, value);

    return value;
  }

  /**
   * Key for the system property defining {@link BASEDIR}
   */
  public static final String BASEDIR_SYS_PROP = "karaf.base";

  /**
   * Key for the system property defining {@link DATADIR}
   */
  public static final String DATADIR_SYS_PROP = "karaf.data";

  /**
   * String representation of the base path of astrum.
   *
   * launch scripts set basedir to "../"
   */
  public static final String BASEDIR = compute(BASEDIR_SYS_PROP, "");

  /**
   * String representation of the data directory (e.g. where we write blobs, logs, etc.)
   */
  public static final String DATADIR = compute(DATADIR_SYS_PROP, "../globalart-work/astrum3");

  public static Path getBasePath(final String... segments) {
    return Paths.get(BASEDIR, segments);
  }

  public static Path getDataPath(final String... segments) {
    return Paths.get(DATADIR, segments);
  }

  /**
   * Ensures the properties are loaded
   */
  public static void load() {
    // no-op we rely on the static constants being triggered during class loading.
  }

  private AstrumDirectoryConfiguration() {
    // private
  }
}
