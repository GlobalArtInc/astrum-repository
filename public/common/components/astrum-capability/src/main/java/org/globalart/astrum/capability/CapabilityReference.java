
package org.globalart.astrum.capability;

/**
 * Reference to a capability and its state.
 *
 * @since capabilities 2.0
 */
public interface CapabilityReference
{

  /**
   * Returns capability context.
   *
   * @return capability context (never null)
   */
  CapabilityContext context();

  /**
   * Returns the referenced capability.
   *
   * @return referenced capability
   */
  Capability capability();

  /**
   * Returns the referenced capability.
   *
   * @param type Capability class type
   * @return referenced capability
   */
  <T extends Capability> T capabilityAs(Class<T> type);

}
