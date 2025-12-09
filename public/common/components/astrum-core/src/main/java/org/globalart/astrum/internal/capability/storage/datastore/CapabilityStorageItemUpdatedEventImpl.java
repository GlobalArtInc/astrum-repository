
package org.globalart.astrum.internal.capability.storage.datastore;

import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemData;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemUpdatedEvent;

public class CapabilityStorageItemUpdatedEventImpl
    extends CapabilityStorageItemEventSupport
    implements CapabilityStorageItemUpdatedEvent
{
  protected CapabilityStorageItemUpdatedEventImpl() {
    // deserialization
  }

  public CapabilityStorageItemUpdatedEventImpl(final CapabilityStorageItemData item) {
    super(item);
  }
}
