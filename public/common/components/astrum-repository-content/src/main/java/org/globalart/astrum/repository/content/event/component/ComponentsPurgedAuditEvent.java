
package org.globalart.astrum.repository.content.event.component;

import java.util.List;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.store.ContentStoreEvent;
import org.globalart.astrum.repository.content.store.internal.ComponentAuditor;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Designed to inform a {@link ComponentAuditor} that some {@link Component}'s from a {@link Repository}, purged and
 * must be recorded in more informative way.
 */
public class ComponentsPurgedAuditEvent
    extends ContentStoreEvent
{
  private final List<Component> components;

  /**
   * Contains a {@link List} of purged components
   *
   * @param repositoryId - not NULL
   * @param components - not NULL, not empty;
   * @throws NullPointerException - if components is null
   */
  public ComponentsPurgedAuditEvent(Integer repositoryId, final List<Component> components) {
    super(checkNotNull(repositoryId));
    this.components = checkNotNull(components);
  }

  public List<Component> getComponents() {
    return components;
  }
}
