
package org.globalart.astrum.repository.content.event.asset;

import java.util.List;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AttributeChangeSet.AttributeChange;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Event sent whenever an {@link Asset}'s attributes change.
 *
 * @since 3.26
 */
public class AssetAttributesEvent
    extends AssetUpdatedEvent
{
  private final List<AttributeChange> changes;

  public AssetAttributesEvent(
      final Asset asset,
      final List<AttributeChange> changes)
  {
    super(asset);
    this.changes = checkNotNull(changes);
  }

  public List<AttributeChange> getChanges() {
    return changes;
  }

  @Override
  public String toString() {
    return "AssetAttributesEvent{" +
        "changes=" + changes +
        "} " + super.toString();
  }
}
