
package org.globalart.astrum.content.raw.internal.search;

import java.util.List;
import java.util.stream.Collectors;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.raw.internal.RawFormat;
import org.globalart.astrum.repository.search.SearchRequest;
import org.globalart.astrum.repository.search.query.SearchFilter;
import org.globalart.astrum.repository.search.sql.SqlSearchQueryContributionSupport;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.search.index.SearchConstants.FORMAT;

/**
 * Routes the user-facing {@value #NAME_RAW} filter to the internal lenient {@link RawSearchMappings#RAW_NAME}
 * alias whenever the search is scoped to {@code format=raw}.
 *
 * <p>
 * Raw stores the full asset path as the component name (e.g. {@code /foo/bar/report.pdf}), so the default
 * exact match against the {@code name} column never succeeds when users type just the basename
 * (e.g. {@code report.pdf}). Rewriting the filter to {@code raw.name} sends the query through the lenient
 * (tsvector) path where {@link RawSearchCustomFieldContributor} has indexed the basename, making filename
 * search work as users expect.
 *
 * <p>
 * No new UI criterion is introduced — the existing {@code name.raw} "Name" field continues to be the
 * user-facing search input across all UIs (React preview UI, Classic ExtJS UI, REST API). Non-raw formats
 * are unaffected because the rewrite is gated on the {@code format=raw} filter being present.
 */
@Component
@Qualifier(NameRawSqlSearchQueryContribution.NAME_RAW)
public class NameRawSqlSearchQueryContribution
    extends SqlSearchQueryContributionSupport
{
  /**
   * The user-facing search criterion property name. Mirrors
   * {@code DefaultSearchMappings.NAME_RAW} without depending on the {@code internal} package.
   */
  public static final String NAME_RAW = "name.raw";

  @Override
  public SearchRequest modifyRequest(final SearchRequest request) {
    if (!isRawFormat(request)) {
      return request;
    }
    List<SearchFilter> rewritten = request.getSearchFilters()
        .stream()
        .map(NameRawSqlSearchQueryContribution::rewriteIfNameRaw)
        .collect(Collectors.toList());
    return SearchRequest.builder()
        .from(request)
        .replaceSearchFilters(rewritten)
        .build();
  }

  private static boolean isRawFormat(final SearchRequest request) {
    return request.getSearchFilters()
        .stream()
        .anyMatch(filter -> filter != null
            && FORMAT.equalsIgnoreCase(filter.getProperty())
            && RawFormat.NAME.equalsIgnoreCase(filter.getValue()));
  }

  private static SearchFilter rewriteIfNameRaw(@Nullable final SearchFilter filter) {
    if (filter == null || !NAME_RAW.equals(filter.getProperty())) {
      return filter;
    }
    return new SearchFilter(RawSearchMappings.RAW_NAME, filter.getValue(), filter.operator());
  }
}
