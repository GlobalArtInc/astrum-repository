
package org.globalart.astrum.internal.capability.storage.datastore;

import org.globalart.astrum.capability.CapabilityIdentity;
import org.globalart.astrum.common.event.EventWithSource;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageImpl;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemData;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemEvent;

import static com.google.common.base.Preconditions.checkNotNull;

public class CapabilityStorageItemEventSupport
    extends EventWithSource
    implements CapabilityStorageItemEvent
{
  private CapabilityIdentity capabilityId;

  protected CapabilityStorageItemEventSupport() {
    // deserialization
  }

  protected CapabilityStorageItemEventSupport(final CapabilityStorageItemData item) {
    this.capabilityId = CapabilityStorageImpl.capabilityIdentity(checkNotNull(item));
  }

  @Override
  public CapabilityIdentity getCapabilityId() {
    return capabilityId;
  }

  public void setCapabilityId(final CapabilityIdentity capabilityId) {
    this.capabilityId = capabilityId;
  }
}
