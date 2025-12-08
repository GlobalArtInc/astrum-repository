
package org.globalart.astrum.internal.capability.storage;

import org.globalart.astrum.capability.CapabilityIdentity;

/**
 * {@link CapabilityStorageItem} event.
 *
 * @since 3.1
 */
public interface CapabilityStorageItemEvent
{
  boolean isLocal();

  CapabilityIdentity getCapabilityId();
}
