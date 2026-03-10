
package org.globalart.astrum.supportzip;

import java.io.File;
import java.io.IOException;

/**
 * Import only Task's related data to a DB.
 *
 * @since 3.30
 */
public interface ImportTaskData
{
  /**
   * Restore serialized tasks data to the DB.
   *
   * @param file file where data will be restored.
   * @throws IOException for any issue during reading a file.
   */
  void restore(final File file) throws IOException;
}
