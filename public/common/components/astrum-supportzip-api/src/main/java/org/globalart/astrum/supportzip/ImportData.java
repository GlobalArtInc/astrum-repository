
package org.globalart.astrum.supportzip;

import java.io.File;
import java.io.IOException;

/**
 * Should be used by the Support ZIP to restore data to a DB.
 *
 * @since 3.29
 */
public interface ImportData
{
  /**
   * Restore serialized data to the DB.
   *
   * @param file file where data will be restored.
   * @throws IOException for any issue during reading a file.
   */
  void restore(final File file) throws IOException;
}
