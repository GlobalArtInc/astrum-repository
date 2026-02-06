
package org.globalart.astrum.repository.content.search.sql;

import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.search.sql.SearchRecord;

/**
 * Allows contribution of data from a component to {@link SearchRecord}
 *
 * This abstraction allows content to be contributed to a {@link SearchRecord}
 * from modules outside the astrum-repository-content (such as the astrum-tags-plugin module)
 *
 */
public interface SearchRecordExtension
{
  void contribute(final SearchRecord searchRecord, final FluentComponent component);
}
