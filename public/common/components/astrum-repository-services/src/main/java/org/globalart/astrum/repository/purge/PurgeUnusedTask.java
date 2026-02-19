
package org.globalart.astrum.repository.purge;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.scheduling.Cancelable;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Task to purge unused components/assets of given repository.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class PurgeUnusedTask
    extends RepositoryTaskSupport
    implements Cancelable
{
  public static final String LAST_USED_FIELD_ID = "lastUsed";

  @Override
  protected void execute(final Repository repository) {
    repository.facet(PurgeUnusedFacet.class).purgeUnused(getConfiguration().getInteger(LAST_USED_FIELD_ID, -1));
  }

  @Override
  protected boolean appliesTo(final Repository repository) {
    return repository.optionalFacet(PurgeUnusedFacet.class).isPresent();
  }

  @Override
  public String getMessage() {
    return "Purge unused components and assets from " + getRepositoryField();
  }
}
