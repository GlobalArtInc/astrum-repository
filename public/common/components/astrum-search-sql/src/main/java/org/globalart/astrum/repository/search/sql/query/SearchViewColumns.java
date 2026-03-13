
package org.globalart.astrum.repository.search.sql.query;

import java.util.Objects;

/**
 * Enum representing column names for {@link SearchDAO}
 *
 * @since 3.38
 */
public enum SearchViewColumns
{
  COMPONENT_ID("componentId"),
  NAMESPACE("namespace"),
  SEARCH_COMPONENT_NAME("componentName"),
  VERSION("version"),
  NORMALISED_VERSION("normalised_version"),
  REPOSITORY_NAME("repositoryName"),
  FORMAT("format");

  private final String name;

  SearchViewColumns(final String name) {
    this.name = name;
  }

  public String getName() {
    return name;
  }

  public static SearchViewColumns fromSortFieldName(final String name) {
    if (Objects.isNull(name)) {
      return COMPONENT_ID;
    }
    switch (name) {
      case "group":
        return NAMESPACE;
      case "id":
        return COMPONENT_ID;
      case "name":
        return SEARCH_COMPONENT_NAME;
      case "version":
        return NORMALISED_VERSION;
      default:
        return COMPONENT_ID;
    }
  }
}
