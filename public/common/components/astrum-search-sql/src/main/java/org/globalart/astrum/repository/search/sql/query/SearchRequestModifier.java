
package org.globalart.astrum.repository.search.sql.query;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.search.SearchRequest;
import org.globalart.astrum.repository.search.query.SearchFilter;
import org.globalart.astrum.repository.search.sql.SqlSearchQueryContribution;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.search.index.SearchConstants.REPOSITORY_NAME;
import org.springframework.stereotype.Component;

@Component
public class SearchRequestModifier
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private Map<String, SqlSearchQueryContribution> handlers;

  @Autowired
  public SearchRequestModifier(final List<SqlSearchQueryContribution> handlersList) {
    this.handlers = QualifierUtil.buildQualifierBeanMap(checkNotNull(handlersList));
  }

  /**
   * Modifies the provided {@link SearchRequest} based on the search filters,
   * excluding the repository name filter. If the filter is not present, or it does not modify the request,
   * the original request is returned.
   */
  public SearchRequest modify(final SearchRequest request) {
    SearchRequest updatedSearchRequest = request;
    for (SearchFilter filter : request.getSearchFilters()) {
      if (filter != null && !REPOSITORY_NAME.equalsIgnoreCase(filter.getProperty())) {
        log.debug("Modifying request for {}", filter);
        updatedSearchRequest = modifyRequest(filter, updatedSearchRequest);
      }
    }
    return updatedSearchRequest;
  }

  /**
   * Modifies the provided {@link SearchRequest} based on the provided {@link SearchFilter}.
   */
  private SearchRequest modifyRequest(final SearchFilter filter, SearchRequest request) {
    SqlSearchQueryContribution queryContribution = Optional.ofNullable(filter.getProperty())
        .map(handlers::get)
        .orElse(handlers.get("default"));
    return queryContribution.modifyRequest(request);
  }
}
