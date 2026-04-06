package org.globalart.astrum.repository.nuget.internal.search;

import java.util.List;

import org.globalart.astrum.repository.nuget.internal.NugetFormat;
import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;

import com.google.common.collect.ImmutableList;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_1;
import static org.globalart.astrum.repository.rest.sql.SearchField.NAME;

@Component
@Qualifier(NugetFormat.NAME)
public class NugetSearchMappings
    implements SearchMappings
{
  private static final List<SearchMapping> MAPPINGS = ImmutableList.of(
      new SearchMapping("nuget.id", "attributes.nuget.id", "NuGet id", NAME, false),
      new SearchMapping("nuget.tags", "assets.attributes.nuget.tags", "NuGet tags", FORMAT_FIELD_1, false));

  @Override
  public Iterable<SearchMapping> get() {
    return MAPPINGS;
  }
}
