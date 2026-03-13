
package org.globalart.astrum.repository.search.sql.query.h2;

import java.util.Optional;

/**
 * Represents a searchable column in H2 database.
 * H2 doesn't support PostgreSQL's TSVECTOR full-text search, so we use simple VARCHAR with LIKE queries.
 *
 * @param columnName the database column name for matches
 * @param sortColumnName the database column name that will be used for sorting
 * @param tokenized whether this column contains tokenized data that always requires LIKE queries
 */
record H2SearchColumn(String columnName, Optional<String> sortColumnName, boolean tokenized, Type type)
{
  /**
   * @param columnName the database column name that will also be used for sorting
   */
  H2SearchColumn(final String columnName) {
    this(columnName, columnName, false);
  }

  /**
   * @param columnName the database column name that will also be used for sorting
   */
  H2SearchColumn(final String columnName, final Type type) {
    this(columnName, columnName, false, type);
  }

  /**
   * @param columnName the database column name for matches
   * @param sortColumnName the database column name that will be used for sorting
   */
  H2SearchColumn(final String columnName, final String sortColumnName) {
    this(columnName, sortColumnName, false);
  }

  /**
   * @param columnName the database column name that will also be used for sorting
   * @param tokenized whether this column contains tokenized data that always requires LIKE queries
   */
  H2SearchColumn(final String columnName, final boolean tokenized) {
    this(columnName, columnName, tokenized);
  }

  /**
   * @param columnName the database column name for matches
   * @param sortColumnName the database column name that will be used for sorting
   * @param tokenized whether this column contains tokenized data that always requires LIKE queries
   */
  H2SearchColumn(final String columnName, final String sortColumnName, final boolean tokenized) {
    this(columnName, sortColumnName, tokenized, Type.VARCHAR);
  }

  /**
   * @param columnName the database column name for matches
   * @param sortColumnName the database column name that will be used for sorting
   * @param tokenized whether this column contains tokenized data that always requires LIKE queries
   */
  H2SearchColumn(final String columnName, final String sortColumnName, final boolean tokenized, final Type type) {
    this(columnName, Optional.ofNullable(sortColumnName), tokenized, type);
  }

  /**
   * @param columnName the database column name for matches
   * @param tokenized whether this column contains tokenized data that always requires LIKE queries
   * @param jsonColumn whether this column contains json data that always requires special handling for exact match
   *          queries
   */
  H2SearchColumn(final String columnName, final boolean tokenized, final Type type) {
    this(columnName, Optional.of(columnName), tokenized, type);
  }

  /**
   * H2 columns support text search via LIKE
   */
  boolean supportsTextSearch() {
    return !isBooleanColumn();
  }

  /**
   * @return whether this column supports exact matching (prefix search for wildcards).
   *         Non-tokenized, non-JSON columns support exact matching.
   */
  boolean supportsExact() {
    return !tokenized && !isJsonColumn();
  }

  /**
   * @return whether this column contains json data that always requires special handling for exact match queries
   */
  boolean isJsonColumn() {
    return Type.JSON.equals(type);
  }

  /**
   * @return whether this column contains json data that always requires special handling for exact match queries
   */
  boolean isBooleanColumn() {
    return Type.BOOLEAN.equals(type);
  }

  enum Type
  {
    VARCHAR,
    BOOLEAN,
    JSON
  }
}
