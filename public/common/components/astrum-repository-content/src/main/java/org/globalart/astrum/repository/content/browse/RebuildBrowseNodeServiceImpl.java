
package org.globalart.astrum.repository.content.browse;

import java.util.function.Consumer;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.browse.node.RebuildBrowseNodeFailedException;
import org.globalart.astrum.repository.browse.node.RebuildBrowseNodeService;
import org.springframework.stereotype.Component;

/**
 * Rebuild browse node service for content repositories.
 *
 * @since 3.26
 */
@Component
public class RebuildBrowseNodeServiceImpl
    implements RebuildBrowseNodeService
{
  @Override
  public void rebuild(
      final Repository repository,
      final Consumer<String> progressUpdater) throws RebuildBrowseNodeFailedException
  {
    repository.optionalFacet(BrowseFacet.class)
        .ifPresent(browseFacet -> browseFacet.rebuildBrowseNodes(progressUpdater));
  }
}
