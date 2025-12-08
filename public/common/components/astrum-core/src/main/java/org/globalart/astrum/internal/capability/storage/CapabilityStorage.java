
package org.globalart.astrum.internal.capability.storage;

import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.common.lifecycle.Lifecycle;
import org.globalart.astrum.capability.CapabilityIdentity;
import org.globalart.astrum.common.entity.EntityId;

/**
 * Capability storage.
 */
public interface CapabilityStorage
    extends Lifecycle
{
  /**
   * Adds a capability
   *
   * @param item to be added
   */
  CapabilityIdentity add(CapabilityStorageItem item);

  // FIXME: update() and remove() return values are never used, probably should return void instead

  /**
   * Updates stored capability if exists.
   *
   * @param item to be updated
   * @return false if capability to be updated does not exist in storage, true otherwise
   */
  boolean update(CapabilityIdentity id, CapabilityStorageItem item);

  /**
   * Deletes stored capability if exists.
   *
   * @param id of capability to be deleted
   * @return false if capability to be deleted does not exist in storage, true otherwise
   */
  boolean remove(CapabilityIdentity id);

  /**
   * Retrieves a single stored capability by its entity ID.
   *
   * @param entityId the entity ID to retrieve
   * @return optional containing the capability storage item, or empty if not found
   */
  Optional<CapabilityStorageItem> read(EntityId entityId);

  /**
   * Retrieves stored capabilities.
   *
   * @return capabilities (never null)
   */
  Map<CapabilityIdentity, CapabilityStorageItem> getAll();

  /**
   * @since 3.20
   */
  CapabilityStorageItem newStorageItem(
      final int version,
      final String type,
      final boolean enabled,
      final String notes,
      final Map<String, String> properties);
}
