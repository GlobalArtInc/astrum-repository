
package org.globalart.astrum.repository.npm.internal.search;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import java.util.List;


import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;

import com.google.common.collect.ImmutableList;

import static org.globalart.astrum.repository.rest.sql.SearchField.NAMESPACE;

/**
 * @since 3.7
 */
@Component
@Qualifier("npm")
public class NpmSearchMappings
    implements SearchMappings
{
  private static final List<SearchMapping> MAPPINGS = ImmutableList.of(
      new SearchMapping("npm.scope", "group.raw", "npm scope", NAMESPACE)
  );

  @Override
  public Iterable<SearchMapping> get() {
    return MAPPINGS;
  }
}
