
package org.globalart.astrum.repository.search.index;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.scheduling.Cancelable;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Internal task to rebuild index of given repository.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RebuildIndexTask
    extends RepositoryTaskSupport
    implements Cancelable
{
  @Override
  protected void execute(final Repository repository) {
    try {
      repository.facet(SearchIndexFacet.class).rebuildIndex();
    }
    catch (Exception e) {
      log.error("Could not perform rebuild index for repo {}, {}", repository.getName(), e.getMessage());
    }
  }

  @Override
  protected boolean appliesTo(final Repository repository) {
    return repository.optionalFacet(SearchIndexFacet.class).isPresent();
  }

  @Override
  public String getMessage() {
    return "Rebuilding search index of " + getRepositoryField();
  }
}
