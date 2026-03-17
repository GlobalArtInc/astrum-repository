
package org.globalart.astrum.content.raw.internal.search;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.search.sql.SearchCustomFieldContributor;
import org.globalart.astrum.repository.raw.internal.RawFormat;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * Populates raw format custom search fields.
 *
 * <p>
 * Raw stores the full asset path as the component name (e.g. {@code /foo/bar/test.txt}). Searches against the existing
 * {@code name} alias therefore only match the full path, which means typing the bare filename (e.g. {@code test.txt})
 * in the UI's "Name" field returns no results. This contributor registers the basename (the portion after the final
 * {@code /}) as an additional component-name alias so the existing {@code name} search matches by filename for the raw
 * format, without introducing a new search criterion.
 */
@Component
@Qualifier(RawFormat.NAME)
public class RawSearchCustomFieldContributor
    implements SearchCustomFieldContributor
{
  @Override
  public void populateSearchCustomFields(final SearchRecord searchRecord, final Asset asset) {
    String basename = extractBasename(asset.path());
    if (basename != null) {
      searchRecord.addAliasComponentName(basename);
    }
  }

  private static String extractBasename(final String path) {
    if (path == null || path.isEmpty()) {
      return null;
    }
    int lastSlash = path.lastIndexOf('/');
    if (lastSlash < 0) {
      return path;
    }
    if (lastSlash == path.length() - 1) {
      return null;
    }
    return path.substring(lastSlash + 1);
  }
}
