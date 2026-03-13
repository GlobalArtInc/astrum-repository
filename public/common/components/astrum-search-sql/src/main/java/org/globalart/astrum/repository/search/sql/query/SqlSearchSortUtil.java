
package org.globalart.astrum.repository.search.sql.query;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.Function;
import java.util.stream.Collectors;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;
import org.globalart.astrum.repository.rest.sql.SearchField;
import org.globalart.astrum.repository.search.SortDirection;
import org.globalart.astrum.repository.search.index.SearchConstants;

import org.apache.commons.lang3.StringUtils;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Optional.empty;
import static java.util.Optional.of;
import static java.util.stream.StreamSupport.stream;
import static org.apache.commons.lang3.StringUtils.indexOf;
import static org.apache.commons.lang3.StringUtils.replace;
import static org.apache.commons.lang3.StringUtils.startsWith;
import static org.apache.commons.lang3.StringUtils.substring;

import org.springframework.stereotype.Component;

/**
 * Utility class for forming a sort expression based on a search field's column mapping
 */
@Component
public class SqlSearchSortUtil
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String MAVEN_2 = "maven2";

  public static final String DOCKER_LAYER_ANCESTRY = "layerAncestry";

  public static final String DOCKER_CONTENT_DIGEST = "content_digest";

  private final SearchDatabase searchDatabase;

  private final Map<String, SearchField> aliasToColumn;

  @Autowired
  public SqlSearchSortUtil(final SearchDatabase searchDatabase, final List<SearchMappings> searchMappings) {
    this.searchDatabase = checkNotNull(searchDatabase);
    this.aliasToColumn = checkNotNull(searchMappings).stream()
        .map(SearchMappings::get)
        .flatMap(iterable -> stream(iterable.spliterator(), false))
        .collect(Collectors.toMap(mapping -> getAlias(mapping.getAlias()), SearchMapping::getField));
  }

  public Optional<String> getSortExpression(@Nullable final String sortAttribute) {
    Optional<String> sortAlias = Optional.ofNullable(sortAttribute)
        .flatMap(this::isAttributesField);

    Optional<String> sortColumn = sortAlias.map(aliasToColumn::get)
        .flatMap(searchDatabase::getSortColumn);

    return sortColumn.filter(SearchDatabase.ATTRIBUTES::equals)
        .map(sortExpressionForAttributesColumn(sortAlias))
        .orElse(sortColumn);
  }

  public Optional<SortDirection> getSortDirection(@Nullable final String sortAttribute) {
    return Optional.ofNullable(sortAttribute)
        .flatMap(this::isAttributesField)
        .map(aliasToColumn::get)
        .map(SearchField::direction);
  }

  private Function<String, Optional<String>> sortExpressionForAttributesColumn(final Optional<String> sortAlias) {
    return column -> sortAlias.map(alias -> searchDatabase.formatJsonPath(column, alias));
  }

  private Optional<String> isAttributesField(final String value) {
    return startsWith(value, "assets.attributes.") ||
        startsWith(value, "attributes.") ? extractSuffix(value) : of(value);
  }

  private Optional<String> extractSuffix(@Nullable final String sortAttribute) {
    if (sortAttribute == null) {
      return empty();
    }

    int prefixIndex = indexOf(sortAttribute, SearchConstants.ATTRIBUTES);
    if (prefixIndex >= 0) {
      int beginIndex = prefixIndex + "attributes.".length();
      return of(substring(sortAttribute, beginIndex));
    }
    return empty();
  }

  protected static String getAlias(final String alias) {
    if (startsWith(alias, "maven.")) {
      return replace(alias, "maven", MAVEN_2);
    }

    if (StringUtils.equals(alias, "docker.layerId")) {
      return replace(alias, "layerId", DOCKER_LAYER_ANCESTRY);
    }

    if (StringUtils.equals(alias, "docker.contentDigest")) {
      return replace(alias, "contentDigest", DOCKER_CONTENT_DIGEST);
    }
    return alias;
  }
}
