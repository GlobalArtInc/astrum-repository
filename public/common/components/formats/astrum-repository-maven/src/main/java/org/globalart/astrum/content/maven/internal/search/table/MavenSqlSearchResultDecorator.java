
package org.globalart.astrum.content.maven.internal.search.table;

import java.util.Map;

import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.search.ComponentSearchResult;
import org.globalart.astrum.repository.search.sql.SearchResult;
import org.globalart.astrum.repository.search.sql.SqlSearchResultDecorator;

import static java.util.Collections.emptyMap;
import static org.globalart.astrum.repository.maven.internal.Attributes.P_BASE_VERSION;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * An {@link SqlSearchResultDecorator} which annotates {@link ComponentSearchResult} with the maven baseVersion.
 */
@Component
@Qualifier(Maven2Format.NAME)
public class MavenSqlSearchResultDecorator
    implements SqlSearchResultDecorator
{
  @Override
  public void updateComponent(final ComponentSearchResult component, final SearchResult searchResult) {
    if (Maven2Format.NAME.equals(component.getFormat())) {
      Object formatAttributes = searchResult.attributes().get(Maven2Format.NAME);

      @SuppressWarnings("unchecked")
      Map<String, String> attributes =
          formatAttributes instanceof Map ? (Map<String, String>) formatAttributes : emptyMap();

      component.addAnnotation(P_BASE_VERSION, attributes.get(P_BASE_VERSION));
    }
  }
}
