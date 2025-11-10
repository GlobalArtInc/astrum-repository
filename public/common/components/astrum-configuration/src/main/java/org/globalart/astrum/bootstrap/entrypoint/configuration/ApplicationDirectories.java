
package org.globalart.astrum.bootstrap.entrypoint.configuration;

import java.io.File;

/**
 * Provides access to application directories.
 *
 * @since 2.8
 */
public interface ApplicationDirectories
{
  /**
   * Installation directory.
   */
  File getInstallDirectory();

  /**
   * Configuration directory.
   *
   * @param subsystem Sub-system name
   */
  File getConfigDirectory(String subsystem);

  /**
   * Temporary directory.
   */
  File getTemporaryDirectory();

  /**
   * Work directory.
   */
  File getWorkDirectory();

  /**
   * Work sub-directory.
   *
   * @param path Sub-directory path.
   * @param create True to create the directory if it does not exist.
   */
  File getWorkDirectory(String path, boolean create);

  /**
   * Work sub-directory.
   */
  File getWorkDirectory(String path);
}
