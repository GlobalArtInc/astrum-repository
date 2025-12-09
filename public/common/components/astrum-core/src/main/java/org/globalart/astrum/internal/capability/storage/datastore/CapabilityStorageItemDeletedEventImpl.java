
package org.globalart.astrum.internal.capability.storage.datastore;

import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemData;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemDeletedEvent;

public class CapabilityStorageItemDeletedEventImpl
    extends CapabilityStorageItemEventSupport
    implements CapabilityStorageItemDeletedEvent
{
  protected CapabilityStorageItemDeletedEventImpl() {
    // deserialization
  }

  public CapabilityStorageItemDeletedEventImpl(final CapabilityStorageItemData item) {
    super(item);
  }
}
