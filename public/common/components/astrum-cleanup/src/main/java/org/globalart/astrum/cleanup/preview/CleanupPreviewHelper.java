
package org.globalart.astrum.cleanup.preview;

import java.util.stream.Stream;

import org.globalart.astrum.cleanup.storage.CleanupPolicyPreviewXO;
import org.globalart.astrum.extdirect.model.PagedResponse;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.query.QueryOptions;
import org.globalart.astrum.repository.rest.api.ComponentXO;

/**
 * Cleanup preview helper.
 *
 * @since 3.24
 */
public interface CleanupPreviewHelper
{
  PagedResponse<ComponentXO> getSearchResults(
      CleanupPolicyPreviewXO previewXO,
      Repository repository,
      QueryOptions queryOptions);

  Stream<ComponentXO> getSearchResultsStream(
      CleanupPolicyPreviewXO previewXO,
      Repository repository,
      QueryOptions queryOptions);
}
