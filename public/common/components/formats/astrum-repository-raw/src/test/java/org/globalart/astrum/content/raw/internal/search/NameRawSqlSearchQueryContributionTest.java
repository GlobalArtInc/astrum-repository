
package org.globalart.astrum.content.raw.internal.search;

import java.util.List;

import org.globalart.astrum.repository.search.SearchRequest;
import org.globalart.astrum.repository.search.query.SearchFilter;
import org.globalart.astrum.repository.search.query.SearchFilter.FilterOperator;

import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class NameRawSqlSearchQueryContributionTest
{
  private final NameRawSqlSearchQueryContribution underTest = new NameRawSqlSearchQueryContribution();

  @Test
  void rewritesNameRawToRawNameWhenFormatIsRaw() {
    SearchRequest request = SearchRequest.builder()
        .searchFilter("format", "raw")
        .searchFilter(NameRawSqlSearchQueryContribution.NAME_RAW, "oci-image-spec.pdf")
        .build();

    SearchRequest modified = underTest.modifyRequest(request);

    List<SearchFilter> filters = modified.getSearchFilters();
    assertThat(filters).hasSize(2);
    assertThat(filters)
        .extracting(SearchFilter::getProperty)
        .containsExactly("format", RawSearchMappings.RAW_NAME);
    assertThat(filters)
        .extracting(SearchFilter::getValue)
        .containsExactly("raw", "oci-image-spec.pdf");
  }

  @Test
  void leavesRequestUnchangedWhenFormatIsNotRaw() {
    SearchRequest request = SearchRequest.builder()
        .searchFilter("format", "maven2")
        .searchFilter(NameRawSqlSearchQueryContribution.NAME_RAW, "commons-lang3")
        .build();

    SearchRequest modified = underTest.modifyRequest(request);

    assertThat(modified.getSearchFilters())
        .extracting(SearchFilter::getProperty)
        .containsExactly("format", NameRawSqlSearchQueryContribution.NAME_RAW);
  }

  @Test
  void leavesRequestUnchangedWhenFormatFilterIsAbsent() {
    SearchRequest request = SearchRequest.builder()
        .searchFilter(NameRawSqlSearchQueryContribution.NAME_RAW, "report.pdf")
        .build();

    SearchRequest modified = underTest.modifyRequest(request);

    assertThat(modified.getSearchFilters())
        .extracting(SearchFilter::getProperty)
        .containsExactly(NameRawSqlSearchQueryContribution.NAME_RAW);
  }

  @Test
  void doesNotTouchOtherFilters() {
    SearchRequest request = SearchRequest.builder()
        .searchFilter("format", "raw")
        .searchFilter("repository_name", "raw-hosted")
        .searchFilter(NameRawSqlSearchQueryContribution.NAME_RAW, "report.pdf")
        .build();

    SearchRequest modified = underTest.modifyRequest(request);

    assertThat(modified.getSearchFilters())
        .extracting(SearchFilter::getProperty)
        .containsExactly("format", "repository_name", RawSearchMappings.RAW_NAME);
  }

  @Test
  void preservesFilterOperator() {
    SearchRequest request = SearchRequest.builder()
        .searchFilter("format", "raw")
        .searchFilter(NameRawSqlSearchQueryContribution.NAME_RAW, "report.pdf", FilterOperator.OR)
        .build();

    SearchRequest modified = underTest.modifyRequest(request);

    SearchFilter rewritten = modified.getSearchFilters()
        .stream()
        .filter(filter -> RawSearchMappings.RAW_NAME.equals(filter.getProperty()))
        .findFirst()
        .orElseThrow();
    assertThat(rewritten.getOperator()).contains(FilterOperator.OR);
  }

  @Test
  void rewritesAllNameRawFiltersWhenFormatIsRaw() {
    SearchRequest request = SearchRequest.builder()
        .searchFilter("format", "raw")
        .searchFilter(NameRawSqlSearchQueryContribution.NAME_RAW, "first.pdf")
        .searchFilter(NameRawSqlSearchQueryContribution.NAME_RAW, "second.pdf")
        .build();

    SearchRequest modified = underTest.modifyRequest(request);

    List<String> rewrittenProperties = modified.getSearchFilters()
        .stream()
        .map(SearchFilter::getProperty)
        .toList();
    assertThat(rewrittenProperties)
        .containsExactly("format", RawSearchMappings.RAW_NAME, RawSearchMappings.RAW_NAME);
  }
}
