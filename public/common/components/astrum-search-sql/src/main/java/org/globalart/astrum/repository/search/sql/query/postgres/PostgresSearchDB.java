package org.globalart.astrum.repository.search.sql.query.postgres;

import java.util.Arrays;
import java.util.Collections;
import java.util.EnumMap;
import java.util.Map;
import java.util.Optional;
import java.util.stream.Collectors;

import org.globalart.astrum.repository.rest.sql.SearchField;
import org.globalart.astrum.repository.search.sql.query.SearchDatabase;
import org.globalart.astrum.repository.search.sql.query.postgres.PostgresSearchColumn.Type;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

import static org.apache.commons.lang3.StringUtils.isBlank;
import static org.globalart.astrum.repository.rest.sql.SearchField.*;

@Lazy
@Qualifier("postgres")
@Component
public class PostgresSearchDB
    implements SearchDatabase
{
  private final Map<SearchField, PostgresSearchColumn> columns;

  public PostgresSearchDB() {
    this.columns = Collections.unmodifiableMap(columns());
  }

  Optional<PostgresSearchColumn> getColumn(final SearchField field) {
    return Optional.ofNullable(columns.get(field));
  }

  @Override
  public Optional<String> getSortColumn(final SearchField field) {
    return Optional.ofNullable(field)
        .map(columns::get)
        .flatMap(PostgresSearchColumn::sortColumnName);
  }

  @Override
  public String formatJsonPath(final String columnName, final String sortAlias) {
    if (isBlank(columnName) || isBlank(sortAlias)) {
      return null;
    }
    String path = Arrays.stream(sortAlias.split("\\."))
        .map(PostgresSearchDB::quoteArrayElement)
        .collect(Collectors.joining(",", "{", "}"));
    return columnName + " #>> '" + path.replace("'", "''") + "'";
  }

  private static String quoteArrayElement(final String element) {
    return '"' + element.replace("\\", "\\\\").replace("\"", "\\\"") + '"';
  }

  private static Map<SearchField, PostgresSearchColumn> columns() {
    EnumMap<SearchField, PostgresSearchColumn> columns = new EnumMap<>(SearchField.class);

    columns.put(FORMAT, PostgresSearchColumn.textSearchable(component("format"), component("tsvector_format"),
        component("format")));
    columns.put(REPOSITORY_NAME, PostgresSearchColumn.plain(component("search_repository_name")));
    columns.put(COMPONENT_ID, PostgresSearchColumn.plain(component("component_id"), Type.INTEGER));
    columns.put(COMPONENT_KIND, PostgresSearchColumn.plain(component("component_kind")));
    columns.put(LAST_MODIFIED, PostgresSearchColumn.plain(component("last_modified"), Type.TIMESTAMP));
    columns.put(NAMESPACE, PostgresSearchColumn.textSearchable(component("namespace"),
        component("tsvector_namespace"), component("namespace")));
    columns.put(NAME, PostgresSearchColumn.textSearchable(component("search_component_name"),
        component("tsvector_search_component_name"), component("search_component_name")));
    columns.put(VERSION, PostgresSearchColumn.textSearchable(component("version"), component("tsvector_version"),
        component("normalised_version")));
    columns.put(PRERELEASE, PostgresSearchColumn.plain(component("prerelease"), Type.BOOLEAN));

    columns.put(KEYWORDS, PostgresSearchColumn.tsvector(component("keywords"), component("search_component_name")));
    columns.put(FORMAT_FIELD_1, PostgresSearchColumn.tsvector(component("format_field_values_1"), ATTRIBUTES));
    columns.put(FORMAT_FIELD_2, PostgresSearchColumn.tsvector(component("format_field_values_2"), ATTRIBUTES));
    columns.put(FORMAT_FIELD_3, PostgresSearchColumn.tsvector(component("format_field_values_3"), ATTRIBUTES));
    columns.put(FORMAT_FIELD_4, PostgresSearchColumn.tsvector(component("format_field_values_4"), ATTRIBUTES));
    columns.put(FORMAT_FIELD_5, PostgresSearchColumn.tsvector(component("format_field_values_5"), ATTRIBUTES));
    columns.put(FORMAT_FIELD_6, PostgresSearchColumn.tsvector(component("format_field_values_6"), ATTRIBUTES));
    columns.put(FORMAT_FIELD_7, PostgresSearchColumn.tsvector(component("format_field_values_7"), ATTRIBUTES));

    columns.put(MD5, PostgresSearchColumn.tsvector(component("md5"), component("md5")));
    columns.put(SHA1, PostgresSearchColumn.tsvector(component("sha1"), component("sha1")));
    columns.put(SHA256, PostgresSearchColumn.tsvector(component("sha256"), component("sha256")));
    columns.put(SHA512, PostgresSearchColumn.tsvector(component("sha512"), component("sha512")));

    columns.put(TAGS, PostgresSearchColumn.tsvector(component("tsvector_tags"), component("tags")));
    columns.put(PATHS, PostgresSearchColumn.tsvector(component("tsvector_paths"), component("paths"))
        .withRegexColumn(component("paths")));
    columns.put(UPLOADERS, PostgresSearchColumn.tsvector(component("uploaders"), component("uploaders")));
    columns.put(UPLOADER_IPS, PostgresSearchColumn.tsvector(component("uploader_ips"), component("uploader_ips")));

    columns.put(ASSET_FORMAT_VALUE_1, PostgresSearchColumn.plain(asset("asset_format_value_1")));
    columns.put(ASSET_FORMAT_VALUE_2, PostgresSearchColumn.plain(asset("asset_format_value_2")));
    columns.put(ASSET_FORMAT_VALUE_3, PostgresSearchColumn.plain(asset("asset_format_value_3")));
    columns.put(ASSET_FORMAT_VALUE_4, PostgresSearchColumn.plain(asset("asset_format_value_4")));
    columns.put(ASSET_FORMAT_VALUE_5, PostgresSearchColumn.plain(asset("asset_format_value_5")));
    columns.put(ASSET_FORMAT_VALUE_6, PostgresSearchColumn.plain(asset("asset_format_value_6")));
    columns.put(ASSET_FORMAT_VALUE_7, PostgresSearchColumn.plain(asset("asset_format_value_7")));
    columns.put(ASSET_FORMAT_VALUE_8, PostgresSearchColumn.plain(asset("asset_format_value_8")));
    columns.put(ASSET_FORMAT_VALUE_9, PostgresSearchColumn.plain(asset("asset_format_value_9")));
    columns.put(ASSET_FORMAT_VALUE_10, PostgresSearchColumn.plain(asset("asset_format_value_10")));
    columns.put(ASSET_FORMAT_VALUE_11, PostgresSearchColumn.plain(asset("asset_format_value_11")));
    columns.put(ASSET_FORMAT_VALUE_12, PostgresSearchColumn.plain(asset("asset_format_value_12")));
    columns.put(ASSET_FORMAT_VALUE_13, PostgresSearchColumn.plain(asset("asset_format_value_13")));
    columns.put(ASSET_FORMAT_VALUE_14, PostgresSearchColumn.plain(asset("asset_format_value_14")));
    columns.put(ASSET_FORMAT_VALUE_15, PostgresSearchColumn.plain(asset("asset_format_value_15")));
    columns.put(ASSET_FORMAT_VALUE_16, PostgresSearchColumn.plain(asset("asset_format_value_16")));
    columns.put(ASSET_FORMAT_VALUE_17, PostgresSearchColumn.plain(asset("asset_format_value_17")));
    columns.put(ASSET_FORMAT_VALUE_18, PostgresSearchColumn.plain(asset("asset_format_value_18")));
    columns.put(ASSET_FORMAT_VALUE_19, PostgresSearchColumn.plain(asset("asset_format_value_19")));
    columns.put(ASSET_FORMAT_VALUE_20, PostgresSearchColumn.plain(asset("asset_format_value_20")));

    return columns;
  }

  private static String component(final String columnName) {
    return COMPONENT_PREFIX_ID + columnName;
  }

  private static String asset(final String columnName) {
    return ASSET_PREFIX_ID + columnName;
  }
}
