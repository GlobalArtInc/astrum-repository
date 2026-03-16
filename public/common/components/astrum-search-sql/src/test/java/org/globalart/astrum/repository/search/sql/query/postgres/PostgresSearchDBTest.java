package org.globalart.astrum.repository.search.sql.query.postgres;

import java.util.Optional;

import org.globalart.astrum.repository.rest.sql.SearchField;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

class PostgresSearchDBTest
{
  private PostgresSearchDB underTest;

  @BeforeEach
  void setup() {
    underTest = new PostgresSearchDB();
  }

  @Test
  void shouldFormatJsonPathForAttributesColumn() {
    assertThat(underTest.formatJsonPath("cs.attributes", "nuget.id"), is("cs.attributes #>> '{\"nuget\",\"id\"}'"));
    assertThat(underTest.formatJsonPath("cs.attributes", "maven2.baseVersion"),
        is("cs.attributes #>> '{\"maven2\",\"baseVersion\"}'"));
  }

  @Test
  void shouldQuoteJsonPathElements() {
    assertThat(underTest.formatJsonPath("cs.attributes", "a'b.c\"d\\e"),
        is("cs.attributes #>> '{\"a''b\",\"c\\\"d\\\\e\"}'"));
  }

  @Test
  void shouldNotFormatBlankJsonPath() {
    assertThat(underTest.formatJsonPath("cs.attributes", ""), nullValue());
    assertThat(underTest.formatJsonPath("cs.attributes", null), nullValue());
  }

  @Test
  void shouldUseAttributesForFormatFieldSorting() {
    assertThat(underTest.getSortColumn(SearchField.FORMAT_FIELD_1), is(Optional.of("cs.attributes")));
    assertThat(underTest.getSortColumn(SearchField.FORMAT_FIELD_7), is(Optional.of("cs.attributes")));
  }

  @Test
  void shouldUsePlainColumnsForSorting() {
    assertThat(underTest.getSortColumn(SearchField.VERSION), is(Optional.of("cs.normalised_version")));
    assertThat(underTest.getSortColumn(SearchField.NAME), is(Optional.of("cs.search_component_name")));
    assertThat(underTest.getSortColumn(SearchField.KEYWORDS), is(Optional.of("cs.search_component_name")));
    assertThat(underTest.getSortColumn(SearchField.REPOSITORY_NAME), is(Optional.of("cs.search_repository_name")));
    assertThat(underTest.getSortColumn(SearchField.ASSET_FORMAT_VALUE_3), is(Optional.of("ap.asset_format_value_3")));
    assertThat(underTest.getSortColumn(null), is(Optional.empty()));
  }

  @Test
  void shouldMapEverySearchField() {
    for (SearchField field : SearchField.values()) {
      assertThat(field.name(), underTest.getColumn(field).isPresent(), is(true));
    }
  }
}
