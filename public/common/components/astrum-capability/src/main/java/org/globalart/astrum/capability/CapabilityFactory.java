
package org.globalart.astrum.capability;

/**
 * Creates instances of capabilities for a specific type.
 *
 * @since capabilities 2.0
 */
public interface CapabilityFactory
{

  /**
   * Creates capability instance with specified id.
   *
   * @return created capability. Must not be null.
   */
  Capability create();

}
