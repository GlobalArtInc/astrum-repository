
package org.globalart.astrum.repository.browse.node;

import java.util.function.Consumer;

import org.globalart.astrum.repository.Repository;

/**
 * Service for rebuilding browse node data.
 *
 * @since 3.22
 */
public interface RebuildBrowseNodeService
{
  /**
   * @param repository {@link Repository} to rebuild browse node data for.
   */
  void rebuild(Repository repository, Consumer<String> progressUpdater) throws RebuildBrowseNodeFailedException;
}
