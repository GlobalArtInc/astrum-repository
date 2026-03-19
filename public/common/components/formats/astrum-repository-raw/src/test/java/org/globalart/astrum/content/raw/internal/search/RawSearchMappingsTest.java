
package org.globalart.astrum.content.raw.internal.search;

import java.util.List;
import java.util.stream.StreamSupport;

import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.sql.SearchField;

import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class RawSearchMappingsTest
{
  private final RawSearchMappings underTest = new RawSearchMappings();

  @Test
  void rawNameMappingIsRegistered() {
    List<SearchMapping> mappings = StreamSupport.stream(underTest.get().spliterator(), false)
        .toList();

    assertThat(mappings).hasSize(1);
    SearchMapping mapping = mappings.get(0);
    assertThat(mapping.getAlias()).isEqualTo(RawSearchMappings.RAW_NAME);
    assertThat(mapping.getAttribute()).isEqualTo(RawSearchMappings.RAW_NAME_ATTRIBUTE);
    assertThat(mapping.getDescription()).isEqualTo("Raw asset filename (basename)");
    assertThat(mapping.getField()).isEqualTo(SearchField.NAME);
  }

  @Test
  void rawNameMappingIsNotExactMatch() {
    SearchMapping mapping = StreamSupport.stream(underTest.get().spliterator(), false)
        .findFirst()
        .orElseThrow();

    assertThat(mapping.isExactMatch()).isFalse();
  }
}
