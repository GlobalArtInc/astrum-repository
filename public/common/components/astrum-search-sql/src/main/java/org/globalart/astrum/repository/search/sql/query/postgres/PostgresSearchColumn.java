package org.globalart.astrum.repository.search.sql.query.postgres;

import java.util.Optional;

record PostgresSearchColumn(
    Optional<String> columnName,
    Optional<String> tsvectorColumnName,
    Optional<String> regexColumnName,
    Optional<String> sortColumnName,
    Type type)
{
  static PostgresSearchColumn plain(final String columnName) {
    return plain(columnName, Type.VARCHAR);
  }

  static PostgresSearchColumn plain(final String columnName, final Type type) {
    return new PostgresSearchColumn(Optional.of(columnName), Optional.empty(), Optional.of(columnName),
        Optional.of(columnName), type);
  }

  static PostgresSearchColumn textSearchable(
      final String columnName,
      final String tsvectorColumnName,
      final String sortColumnName)
  {
    return new PostgresSearchColumn(Optional.of(columnName), Optional.of(tsvectorColumnName),
        Optional.of(columnName), Optional.of(sortColumnName), Type.VARCHAR);
  }

  static PostgresSearchColumn tsvector(final String tsvectorColumnName, final String sortColumnName) {
    return new PostgresSearchColumn(Optional.empty(), Optional.of(tsvectorColumnName), Optional.empty(),
        Optional.ofNullable(sortColumnName), Type.VARCHAR);
  }

  PostgresSearchColumn withRegexColumn(final String regexColumnName) {
    return new PostgresSearchColumn(columnName, tsvectorColumnName, Optional.of(regexColumnName), sortColumnName,
        type);
  }

  boolean hasColumn() {
    return columnName.isPresent();
  }

  boolean supportsTextSearch() {
    return tsvectorColumnName.isPresent();
  }

  boolean isText() {
    return type == Type.VARCHAR;
  }

  String nullCheckColumn() {
    return columnName.or(() -> tsvectorColumnName).orElseThrow();
  }

  String describe() {
    return columnName.or(() -> tsvectorColumnName).orElse("?");
  }

  String cast(final String placeholder) {
    return switch (type) {
      case INTEGER -> "CAST(" + placeholder + " AS INTEGER)";
      case TIMESTAMP -> "CAST(" + placeholder + " AS TIMESTAMP WITH TIME ZONE)";
      default -> placeholder;
    };
  }

  enum Type
  {
    VARCHAR,
    BOOLEAN,
    INTEGER,
    TIMESTAMP
  }
}
