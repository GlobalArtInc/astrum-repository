
package org.globalart.astrum.repository.rest.internal;

import java.util.Collection;
import java.util.List;
import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;
import org.globalart.astrum.repository.rest.SearchMappingsService;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableList.Builder;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

@Component
public class SearchMappingsServiceImpl
    implements SearchMappingsService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String DEFAULT = "default";

  private final Collection<SearchMapping> searchMappings;

  @Autowired
  public SearchMappingsServiceImpl(final List<SearchMappings> searchMappingsList) {
    this.searchMappings = collectMappings(QualifierUtil.buildQualifierBeanMap(checkNotNull(searchMappingsList)));
  }

  private static Collection<SearchMapping> collectMappings(final Map<String, SearchMappings> searchMappings) {
    final Builder<SearchMapping> builder = ImmutableList.builder();

    // put the default mappings in first
    final SearchMappings defaultMappings = searchMappings.get(DEFAULT);
    if (defaultMappings != null) {
      builder.addAll(defaultMappings.get());
    }

    // add the rest of the mappings
    searchMappings.keySet()
        .stream()
        .filter(key -> !DEFAULT.equals(key))
        .sorted()
        .forEach(key -> builder.addAll(searchMappings.get(key).get()));

    return builder.build();
  }

  @Override
  public Collection<SearchMapping> getAllMappings() {
    return searchMappings;
  }
}
