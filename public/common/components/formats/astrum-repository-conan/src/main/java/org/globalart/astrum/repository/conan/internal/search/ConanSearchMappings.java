
package org.globalart.astrum.repository.conan.internal.search;

import java.util.List;

import org.globalart.astrum.repository.conan.ConanFormat;
import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;

import com.google.common.collect.ImmutableList;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_1;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_2;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_3;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_4;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_5;

@Component
@Qualifier(ConanFormat.NAME)
public class ConanSearchMappings
    implements SearchMappings
{
  private static final List<SearchMapping> MAPPINGS = ImmutableList.of(
      new SearchMapping("conan.baseVersion", "attributes.conan.baseVersion", "Conan base version", FORMAT_FIELD_1,
          false),
      new SearchMapping("conan.channel", "attributes.conan.channel", "Conan channel", FORMAT_FIELD_2, false),
      new SearchMapping("conan.revision", "attributes.conan.revision", "Conan recipe revision", FORMAT_FIELD_3,
          false),
      new SearchMapping("conan.packageId", "assets.attributes.conan.packageId", "Conan package id", FORMAT_FIELD_4,
          false),
      new SearchMapping("conan.packageRevision", "assets.attributes.conan.packageRevision",
          "Conan package revision", FORMAT_FIELD_5, false));

  @Override
  public Iterable<SearchMapping> get() {
    return MAPPINGS;
  }
}
