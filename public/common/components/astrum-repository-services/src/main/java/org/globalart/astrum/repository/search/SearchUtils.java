
package org.globalart.astrum.repository.search;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Optional;
import java.util.Set;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.core.UriInfo;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;
import org.globalart.astrum.repository.rest.api.RepositoryManagerRESTAdapter;
import org.globalart.astrum.repository.search.query.SearchFilter;
import org.globalart.astrum.repository.search.query.SearchFilter.FilterOperator;

import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.stream.Collectors.toList;
import static java.util.stream.Collectors.toMap;
import static java.util.stream.StreamSupport.stream;

/**
 * @since 3.38
 */
@Component
public class SearchUtils
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String CONTINUATION_TOKEN = "continuationToken";

  public static final String SORT_FIELD = "sort";

  public static final String SORT_DIRECTION = "direction";

  private static final String ASSET_PREFIX = "assets.";

  private final RepositoryManagerRESTAdapter repoAdapter;

  private final Map<String, String> searchParams;

  private final Map<String, String> assetSearchParams;

  private final Set<String> assetSearchParamValues;

  @Autowired
  public SearchUtils(
      final RepositoryManagerRESTAdapter repoAdapter,
      final List<SearchMappings> searchMappingsList)
  {
    this.repoAdapter = checkNotNull(repoAdapter);
    this.searchParams = QualifierUtil.buildQualifierBeanMap(checkNotNull(searchMappingsList))
        .entrySet()
        .stream()
        .flatMap(e -> stream(e.getValue().get().spliterator(), true))
        .collect(toMap(SearchMapping::getAlias, SearchMapping::getAttribute));
    this.assetSearchParams = searchParams.entrySet()
        .stream()
        .filter(e -> e.getValue().startsWith(ASSET_PREFIX))
        .collect(toMap(Entry::getKey, Entry::getValue));
    this.assetSearchParamValues = Set.copyOf(assetSearchParams.values());
  }

  public Map<String, String> getSearchParameters() {
    return searchParams;
  }

  public Map<String, String> getAssetSearchParameters() {
    return assetSearchParams;
  }

  public Repository getRepository(final String repository) {
    return repoAdapter.getReadableRepository(repository);
  }

  /**
   * Builds a collection of {@link SearchFilter} based on configured search parameters.
   *
   * @param uriInfo {@link UriInfo} to extract query parameters from
   * @return
   */
  public List<SearchFilter> getSearchFilters(final UriInfo uriInfo) {
    return convertParameters(uriInfo, Arrays.asList(CONTINUATION_TOKEN, SORT_FIELD, SORT_DIRECTION));
  }

  /**
   * Builds a collection of {@link SearchFilter} based on configured search parameters,
   * excluding asset-level parameters. Asset-level parameters (those configured in
   * assetSearchParams) should only be used for post-query filtering of assets, not for
   * the main component search query.
   *
   * @param uriInfo {@link UriInfo} to extract query parameters from
   * @return list of search filters excluding asset-level parameters
   */
  public List<SearchFilter> getComponentSearchFilters(final UriInfo uriInfo) {
    return convertParameters(uriInfo, Arrays.asList(CONTINUATION_TOKEN, SORT_FIELD, SORT_DIRECTION))
        .stream()
        .filter(filter -> !assetSearchParamValues.contains(filter.getProperty()))
        .toList();
  }

  private List<SearchFilter> convertParameters(final UriInfo uriInfo, final List<String> keys) {
    List<SearchFilter> filters = uriInfo.getQueryParameters()
        .entrySet()
        .stream()
        .filter(entry -> !keys.contains(entry.getKey()))
        .flatMap(entry -> {
          String key = searchParams.getOrDefault(entry.getKey(), entry.getKey());
          List<String> values = entry.getValue();
          // When multiple format values (e.g. format=maven2&format=npm), use OR so we match ANY format
          boolean useOr = "format".equals(key) && values.size() > 1;
          return values.stream()
              .map(value -> useOr
                  ? new SearchFilter(key, value, FilterOperator.OR)
                  : new SearchFilter(key, value));
        })
        .collect(toList());

    // Auto-inject format filter if format-specific field is used but format parameter is not present
    Optional<String> detectedFormat = detectFormatFromFields(uriInfo);
    if (detectedFormat.isPresent() && !hasFormatFilter(filters)) {
      log.debug("Auto-injecting format filter: format={}", detectedFormat.get());
      filters = new ArrayList<>(filters);
      filters.add(new SearchFilter("format", detectedFormat.get()));
    }

    return filters;
  }

  /**
   * Detects if any format-specific field is used in the query parameters.
   * Format-specific fields follow the pattern: {format}.{fieldname}
   * Examples: swift.scope, maven.groupId, npm.author
   *
   * @param uriInfo the URI info containing query parameters
   * @return the detected format, or empty if no format-specific field is found
   */
  private Optional<String> detectFormatFromFields(final UriInfo uriInfo) {
    Optional<String> name = uriInfo.getQueryParameters()
        .keySet()
        .stream()
        .filter(searchParams::containsKey)
        .filter(param -> param.contains("."))
        .map(param -> param.substring(0, param.indexOf('.')))
        .findFirst();
    if (name.isPresent() && name.get().equalsIgnoreCase("maven")) {
      name = Optional.of("maven2");
    }
    return name;
  }

  /**
   * Checks if a format filter is already present in the list of filters.
   *
   * @param filters the list of search filters
   * @return true if a format filter exists, false otherwise
   */
  private boolean hasFormatFilter(final List<SearchFilter> filters) {
    return filters.stream()
        .anyMatch(filter -> "format".equals(filter.getProperty()));
  }

  public boolean isAssetSearchParam(final String assetSearchParam) {
    return assetSearchParams.containsKey(assetSearchParam) || isFullAssetAttributeName(assetSearchParam);
  }

  public boolean isFullAssetAttributeName(final String assetSearchParam) {
    return assetSearchParam.startsWith(ASSET_PREFIX);
  }

  public String getFullAssetAttributeName(final String key) {
    return isFullAssetAttributeName(key) ? key : getAssetSearchParameters().get(key);
  }
}
