
package org.globalart.astrum.content.raw.internal.search;

import java.util.List;

import org.globalart.astrum.repository.raw.internal.RawFormat;
import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;

import com.google.common.collect.ImmutableList;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.rest.sql.SearchField.NAME;

/**
 * Raw format search mappings.
 *
 * <p>
 * Registers an internal {@code raw.name} alias that performs a lenient (non-exact) search against the component name
 * field. This alias is <strong>not</strong> exposed in the UI; the user-facing criterion remains {@code name.raw}
 * across all UIs. {@link NameRawSqlSearchQueryContribution} rewrites {@code name.raw} filters to this {@code raw.name}
 * routing target when the request is scoped to {@code format=raw}.
 *
 * <p>
 * Raw stores the full asset path as the component name (e.g. {@code /foo/bar/test.txt}), but
 * {@link RawSearchCustomFieldContributor} also registers the basename as an alias in the fulltext index.
 * Using {@code exactMatch=false} causes the search to go through the fulltext (tsvector) path, where the
 * basename alias is available, so that searching by filename (e.g. {@code oci-image-spec.pdf}) works correctly.
 */
@Component
@Qualifier(RawFormat.NAME)
public class RawSearchMappings
    implements SearchMappings
{
  public static final String RAW_NAME = "raw.name";

  /**
   * The attribute path registered with {@link org.globalart.astrum.repository.rest.SearchMapping} for the
   * {@link #RAW_NAME} alias. This is a routing key used by {@code SearchMappingService} to dispatch REST search
   * parameters — no actual {@code raw.name} attribute is stored on component/asset records during ingestion.
   * Basename matching works via the fulltext alias registered by {@link RawSearchCustomFieldContributor}, not
   * through a stored attribute value.
   */
  public static final String RAW_NAME_ATTRIBUTE = "attributes.raw.name";

  private static final List<SearchMapping> MAPPINGS = ImmutableList.of(
      new SearchMapping(RAW_NAME, RAW_NAME_ATTRIBUTE, "Raw asset filename (basename)", NAME, false));

  @Override
  public Iterable<SearchMapping> get() {
    return MAPPINGS;
  }
}
