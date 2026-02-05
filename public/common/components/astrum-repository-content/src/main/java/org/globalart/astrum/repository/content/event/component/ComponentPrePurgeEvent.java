
package org.globalart.astrum.repository.content.event.component;

import java.util.Arrays;

import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.store.ContentStoreEvent;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Event sent just before a large number of {@link Component}s are purged along with their assets.
 *
 * @since 3.27
 */
public class ComponentPrePurgeEvent
    extends ContentStoreEvent
{
  private final int[] componentIds;

  public ComponentPrePurgeEvent(final int contentRepositoryId, final int[] componentIds) { // NOSONAR
    super(contentRepositoryId);
    this.componentIds = checkNotNull(componentIds);
  }

  public int[] getComponentIds() {
    return componentIds; // NOSONAR
  }

  @Override
  public String toString() {
    return "ComponentPrePurgeEvent{" +
        "componentIds=" + Arrays.toString(componentIds) +
        "} " + super.toString();
  }
}
