package org.globalart.astrum.repository.rubygems.internal.search;

import java.util.List;

import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;
import org.globalart.astrum.repository.rubygems.RubygemsFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_1;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_2;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_3;

@Component
@Qualifier(RubygemsFormat.NAME)
public class RubygemsSearchMappings
    implements SearchMappings
{
  private static final List<SearchMapping> MAPPINGS = List.of(
      new SearchMapping("rubygems.platform", "assets.attributes.rubygems.platform", "RubyGems platform",
          FORMAT_FIELD_1, false),
      new SearchMapping("rubygems.summary", "assets.attributes.rubygems.summary", "RubyGems summary",
          FORMAT_FIELD_2, false),
      new SearchMapping("rubygems.description", "assets.attributes.rubygems.description", "RubyGems description",
          FORMAT_FIELD_3, false));

  @Override
  public Iterable<SearchMapping> get() {
    return MAPPINGS;
  }
}
