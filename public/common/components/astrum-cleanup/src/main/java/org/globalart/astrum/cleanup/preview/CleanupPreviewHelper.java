
package org.globalart.astrum.cleanup.preview;

import java.util.stream.Stream;

import javax.annotation.Nullable;

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

  /**
   * Counts every component the policy described by {@code previewXO} would delete from {@code repository}, together
   * with their assets and the size of the assets' blobs. A policy without criteria deletes nothing. When the work
   * outlasts the configured time limit the totals found so far are returned, marked incomplete. Criteria the preview
   * cannot evaluate are reported as a validation error.
   *
   * @param filter optional text the component's name, group or version must contain, as in the paged preview
   */
  CleanupPreviewSummaryXO getSummary(
      CleanupPolicyPreviewXO previewXO,
      Repository repository,
      @Nullable String filter);
}
