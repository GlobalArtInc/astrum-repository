package org.globalart.astrum.repository.yum.internal.search;

import java.util.List;

import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;
import org.globalart.astrum.repository.yum.YumFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_1;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_4;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_6;
import static org.globalart.astrum.repository.rest.sql.SearchField.NAME;

@Component
@Qualifier(YumFormat.NAME)
public class YumSearchMappings
    implements SearchMappings
{
  private static final List<SearchMapping> MAPPINGS = List.of(
      new SearchMapping("yum.name", "attributes.yum.name", "RPM package name", NAME, false),
      new SearchMapping("yum.version", "assets.attributes.yum.version", "RPM version without release",
          FORMAT_FIELD_1, false),
      new SearchMapping("yum.release", "assets.attributes.yum.release", "RPM release", FORMAT_FIELD_4, false),
      new SearchMapping("yum.architecture", "assets.attributes.yum.architecture", "RPM architecture",
          FORMAT_FIELD_6, false));

  @Override
  public Iterable<SearchMapping> get() {
    return MAPPINGS;
  }
}
