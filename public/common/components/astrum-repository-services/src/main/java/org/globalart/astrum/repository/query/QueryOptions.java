
package org.globalart.astrum.repository.query;

import java.util.Arrays;
import java.util.List;
import java.util.Locale;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkArgument;

/**
 * Query data carrier with fields commonly used when browsing through database and search results. Also does
 * a quick check on the sortProperty and sortDirection fields as defensive programming against SQL injection.
 *
 * @since 3.1
 */
public class QueryOptions
{
  private static final List<String> SORT_PROPERTIES = Arrays.asList("name", "group", "version", "id");

  private static final List<String> SORT_DIRECTIONS = Arrays.asList("asc", "desc");

  private final String filter;

  private final String sortProperty;

  private final String sortDirection;

  private final Integer start;

  private final Integer limit;

  private final String lastId;

  private final boolean contentAuth;

  public QueryOptions(
      @Nullable final String filter,
      @Nullable final String sortProperty,
      @Nullable final String sortDirection,
      @Nullable final Integer start,
      @Nullable final Integer limit)
  {
    this(filter, sortProperty, sortDirection, start, limit, null, true);
  }

  public QueryOptions(
      @Nullable final String filter,
      @Nullable final String sortProperty,
      @Nullable final String sortDirection,
      @Nullable final Integer start,
      @Nullable final Integer limit,
      @Nullable final String lastId)
  {
    this(filter, sortProperty, sortDirection, start, limit, lastId, true);
  }

  public QueryOptions(
      @Nullable final String filter,
      @Nullable final String sortProperty,
      @Nullable final String sortDirection,
      @Nullable final Integer start,
      @Nullable final Integer limit,
      @Nullable final String lastId,
      final boolean contentAuth)
  {
    this.lastId = lastId;
    checkArgument(sortProperty == null || SORT_PROPERTIES.contains(sortProperty.toLowerCase(Locale.ENGLISH)));
    checkArgument(sortDirection == null || SORT_DIRECTIONS.contains(sortDirection.toLowerCase(Locale.ENGLISH)));
    this.filter = filter;
    this.sortProperty = sortProperty;
    this.sortDirection = sortDirection;
    this.start = start;
    this.limit = limit;
    this.contentAuth = contentAuth;
  }

  @Nullable
  public String getFilter() {
    return filter;
  }

  @Nullable
  public String getSortProperty() {
    return sortProperty;
  }

  @Nullable
  public String getSortDirection() {
    return sortDirection;
  }

  @Nullable
  public Integer getStart() {
    return start;
  }

  @Nullable
  public Integer getLimit() {
    return limit;
  }

  @Nullable
  public String getLastId() {
    return lastId;
  }

  public boolean getContentAuth() {
    return contentAuth;
  }
}
