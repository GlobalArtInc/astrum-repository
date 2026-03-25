
package org.globalart.astrum.repository.maven.internal.search;

import org.globalart.astrum.repository.search.BlankValueSearchQueryFilter;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Maven classifier attribute should be ignored from the search request if it is empty.
 */
@Component
@Qualifier("assets.attributes.maven2.classifier")
public class MavenClassifierBlankValueFilter
    implements BlankValueSearchQueryFilter
{
  @Override
  public boolean shouldHandleBlankValue() {
    return false;
  }
}
