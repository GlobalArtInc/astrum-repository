
package org.globalart.astrum.supportzip.datastore;

import java.nio.file.Path;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Restore Helper class.
 *
 * @since 3.30
 */
@Component
public class RestoreHelper
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String FILE_SUFFIX = ".json";

  private static final String DB_FOLDER_NAME = "db";

  private final Path dbPath;

  @Autowired
  public RestoreHelper(final ApplicationDirectories applicationDirectories) {
    this.dbPath = checkNotNull(applicationDirectories).getWorkDirectory(DB_FOLDER_NAME, false).toPath();
  }

  /**
   * Get the absolute path to the DB directory.
   * 
   * @return the DB path
   */
  public Path getDbPath() {
    return dbPath;
  }
}
