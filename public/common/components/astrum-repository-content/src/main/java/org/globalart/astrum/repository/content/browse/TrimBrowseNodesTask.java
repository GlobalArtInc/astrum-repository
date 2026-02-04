
package org.globalart.astrum.repository.content.browse;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.scheduling.Cancelable;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class TrimBrowseNodesTask
    extends RepositoryTaskSupport
    implements Cancelable
{
  @Autowired
  public TrimBrowseNodesTask() {
  }

  @Override
  public String getMessage() {
    return "Trimming empty browse nodes for " + getRepositoryField();
  }

  @Override
  protected void execute(final Repository repository) {
    repository.optionalFacet(BrowseFacet.class).ifPresent(browseFacet -> {
      log.info("Trimming empty browse nodes for repository: {}", repository.getName());
      browseFacet.trimBrowseNodes();
      log.info("Finished trimming empty browse nodes for repository: {}", repository.getName());
    });
  }

  @Override
  protected boolean appliesTo(final Repository repository) {
    return repository != null && repository.optionalFacet(BrowseFacet.class).isPresent();
  }
}
