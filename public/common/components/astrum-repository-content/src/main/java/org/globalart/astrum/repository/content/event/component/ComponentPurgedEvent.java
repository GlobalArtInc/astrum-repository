
package org.globalart.astrum.repository.content.event.component;

import java.util.Arrays;
import java.util.List;

import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.store.ContentStoreEvent;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Event sent whenever a large number of {@link Component}s are purged along with their assets.
 *
 * @since 3.26
 */
public class ComponentPurgedEvent
    extends ContentStoreEvent
{
  private final int[] componentIds;

  private final List<FluentAsset> assets;

  public ComponentPurgedEvent(final int contentRepositoryId, final int[] componentIds) { // NOSONAR
    super(contentRepositoryId);
    this.componentIds = checkNotNull(componentIds);
    this.assets = null;
  }

  public ComponentPurgedEvent(
      final int contentRepositoryId,
      final int[] componentIds,
      final List<FluentAsset> assets)
  {
    super(contentRepositoryId);
    this.componentIds = checkNotNull(componentIds);
    this.assets = checkNotNull(assets);
  }

  public int[] getComponentIds() {
    return componentIds; // NOSONAR
  }

  public List<FluentAsset> getAssets() {
    return assets;
  }

  @Override
  public String toString() {
    return "ComponentPurgedEvent{" +
        "componentIds=" + Arrays.toString(componentIds) +
        "} " + super.toString();
  }
}
