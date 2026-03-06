
package org.globalart.astrum.cleanup.content.search;

import org.globalart.astrum.repository.Format;

public interface CleanupBrowseServiceFactory
{
  /**
   * @param format to browse
   * @return the browse service to use
   */
  CleanupComponentBrowse get(Format format);

  CleanupComponentBrowse getPreviewService();
}
