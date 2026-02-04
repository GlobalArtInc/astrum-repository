
package org.globalart.astrum.repository.content;

import javax.annotation.Nullable;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.manager.RepositoryAttributeService;

import org.springframework.stereotype.Component;

@Component
public class RepositoryAttributeServiceImpl
    implements RepositoryAttributeService
{
  @Override
  public boolean setRepositoryAttribute(final Repository repository, final String key, final Object value) {
    try {
      repository.facet(ContentFacet.class).withAttribute(key, value);
      return true;
    }
    catch (Exception e) {
      return false;
    }
  }

  @Override
  @Nullable
  public String getRepositoryAttribute(
      final Repository repository,
      final String key,
      @Nullable final String defaultValue)
  {
    try {
      return repository.facet(ContentFacet.class).attributes().get(key, String.class);
    }
    catch (Exception e) {
      return defaultValue;
    }
  }
}
