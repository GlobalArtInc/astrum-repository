
package org.globalart.astrum.internal.capability.storage.datastore;

import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemCreatedEvent;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemData;

public class CapabilityStorageItemCreatedEventImpl
    extends CapabilityStorageItemEventSupport
    implements CapabilityStorageItemCreatedEvent
{
  protected CapabilityStorageItemCreatedEventImpl() {
    // deserialization
  }

  public CapabilityStorageItemCreatedEventImpl(final CapabilityStorageItemData item) {
    super(item);
  }
}
