
package org.globalart.astrum.repository.search.internal;

import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.stream.Collectors;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.search.ComponentSearchResult;
import org.globalart.astrum.repository.search.SearchResponse;
import org.globalart.astrum.repository.search.query.SearchResultComponentGenerator;
import org.globalart.astrum.repository.search.query.SearchResultsGenerator;
import org.springframework.stereotype.Component;

/**
 * Generates search results consumable by the UI
 *
 * @since 3.14
 */
@Component
public class SearchResultsGeneratorImpl
    implements SearchResultsGenerator
{

  private final Map<String, SearchResultComponentGenerator> searchResultComponentGeneratorMap;

  private final SearchResultComponentGenerator defaultSearchResultComponentGenerator;

  @Autowired
  SearchResultsGeneratorImpl(
      final List<SearchResultComponentGenerator> searchResultComponentGeneratorList)
  {
    this.searchResultComponentGeneratorMap = QualifierUtil.buildQualifierBeanMap(searchResultComponentGeneratorList);
    this.defaultSearchResultComponentGenerator = this.searchResultComponentGeneratorMap
        .get(DefaultSearchResultComponentGenerator.DEFAULT_SEARCH_RESULT_COMPONENT_GENERATOR_KEY);
  }

  @Override
  public List<ComponentSearchResult> getSearchResultList(final SearchResponse response) {
    Set<String> componentIdSet = new HashSet<>();

    return response.getSearchResults()
        .stream()
        .map(component -> {
          String format = component.getFormat();

          SearchResultComponentGenerator generator = searchResultComponentGeneratorMap
              .getOrDefault(format, defaultSearchResultComponentGenerator);

          ComponentSearchResult hit = generator.from(component, componentIdSet);
          if (null != hit) {
            componentIdSet.add(hit.getId());
          }
          return hit;
        })
        .filter(Objects::nonNull)
        .collect(Collectors.toList());
  }
}
