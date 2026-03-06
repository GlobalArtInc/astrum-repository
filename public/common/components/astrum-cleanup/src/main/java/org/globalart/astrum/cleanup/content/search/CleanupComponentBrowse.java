
package org.globalart.astrum.cleanup.content.search;

import java.util.stream.Stream;

import org.globalart.astrum.cleanup.storage.CleanupPolicy;
import org.globalart.astrum.extdirect.model.PagedResponse;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.query.QueryOptions;

/**
 * Finds components to be cleaned up.
 *
 * @since 3.14
 */
public interface CleanupComponentBrowse
{
  Stream<FluentComponent> browse(CleanupPolicy policy, Repository repository);

  Stream<FluentComponent> browseIncludingAssets(CleanupPolicy policy, Repository repository);

  /**
   * Returns a paged response of components
   *
   * @param policy cleanup policy criteria
   * @param repository repository to browse
   * @param options specifies paging and sorting criteria
   * @return a page of {@link Component}
   */
  PagedResponse<Component> browseByPage(CleanupPolicy policy, Repository repository, QueryOptions options);
}
