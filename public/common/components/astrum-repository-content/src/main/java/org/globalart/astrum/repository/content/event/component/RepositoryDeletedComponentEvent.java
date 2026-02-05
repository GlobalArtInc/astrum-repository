
package org.globalart.astrum.repository.content.event.component;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.store.ContentStoreEvent;

/**
 * Event sent when {@link Component}s are deleted during repository deletion.
 */
public class RepositoryDeletedComponentEvent
    extends ContentStoreEvent
{
  private final int deletedComponentsAmount;

  public RepositoryDeletedComponentEvent(final int repositoryId, final int deletedComponentsAmount) {
    super(repositoryId);
    this.deletedComponentsAmount = deletedComponentsAmount;
  }

  public int getDeletedComponentsAmount() {
    return deletedComponentsAmount;
  }

  @Override
  public String toString() {
    Repository repository = getRepository().orElse(null);
    return "RepositoryDeletedComponentEvent{" +
        "repository=" + repository +
        ", deletedComponentsAmount=" + deletedComponentsAmount +
        "}";
  }
}
