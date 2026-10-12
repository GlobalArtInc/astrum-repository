package org.globalart.astrum.repository.swift.internal.search;

import java.util.List;

import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;
import org.globalart.astrum.repository.swift.SwiftFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.rest.sql.SearchField.NAME;
import static org.globalart.astrum.repository.rest.sql.SearchField.NAMESPACE;
import static org.globalart.astrum.repository.rest.sql.SearchField.VERSION;

@Component
@Qualifier(SwiftFormat.NAME)
public class SwiftSearchMappings
    implements SearchMappings
{
  private static final List<SearchMapping> MAPPINGS = List.of(
      new SearchMapping("swift.scope", "attributes.swift.scope", "Swift package scope", NAMESPACE),
      new SearchMapping("swift.name", "attributes.swift.name", "Swift package name", NAME),
      new SearchMapping("swift.version", "attributes.swift.version", "Swift package release version", VERSION));

  @Override
  public Iterable<SearchMapping> get() {
    return MAPPINGS;
  }
}
