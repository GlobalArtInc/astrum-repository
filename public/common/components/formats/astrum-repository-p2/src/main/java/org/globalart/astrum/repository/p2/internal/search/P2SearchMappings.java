
package org.globalart.astrum.repository.p2.internal.search;

import java.util.List;

import org.globalart.astrum.repository.p2.internal.P2Format;
import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;

import com.google.common.collect.ImmutableList;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_1;

@Component
@Qualifier(P2Format.NAME)
public class P2SearchMappings
    implements SearchMappings
{
  private static final List<SearchMapping> MAPPINGS = ImmutableList.of(
      new SearchMapping("p2.pluginName", "attributes.p2.pluginName", "p2 plugin name", FORMAT_FIELD_1, false));

  @Override
  public Iterable<SearchMapping> get() {
    return MAPPINGS;
  }
}
