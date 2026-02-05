
package org.globalart.astrum.repository.content.importtask;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;

/**
 * Provides some post processing capabilities for assets on the import task
 *
 * @since 3.29
 */
public interface ImportPostProcessor
{
  void attributePostProcessing(Repository repository, Asset asset);
}
