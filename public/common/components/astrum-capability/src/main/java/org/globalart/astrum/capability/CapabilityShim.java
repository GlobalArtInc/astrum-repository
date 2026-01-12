
package org.globalart.astrum.capability;

import java.util.Collection;

/**
 * Decouples the capabilities REST resource from concrete capability-shim implementations, allowing
 * the resource to delegate capability operations to any registered shim without depending on
 * shim-specific types (e.g. firewall, IQ-server-connection-info).
 * <p>
 * A shim presents a synthetic capability API for state that has been migrated out of the
 * capability registry into another storage layer (typically repository configuration). The shim is
 * responsible for translating the legacy capability-shaped request into the new model and back.
 */
public interface CapabilityShim
{
  /**
   * Returns {@code true} if this shim handles capabilities of the given type ID.
   */
  boolean handlesType(String typeId);

  /**
   * Returns {@code true} if this shim handles the capability with the given ID.
   */
  boolean handlesId(String capabilityId);

  /**
   * Returns the synthetic {@link CapabilityTypeDTO} exposed by this shim.
   */
  CapabilityTypeDTO getCapabilityTypeDTO();

  /**
   * Returns all synthetic capability DTOs managed by this shim.
   */
  Collection<CapabilityDTO> list();

  /**
   * Creates a capability from the given DTO and returns the resulting synthetic DTO.
   */
  CapabilityDTO create(CapabilityDTO dto);

  /**
   * Updates the capability identified by {@code capabilityId} using the given DTO.
   */
  void update(String capabilityId, CapabilityDTO dto);

  /**
   * Deletes the capability identified by {@code capabilityId}.
   */
  void delete(String capabilityId);
}
