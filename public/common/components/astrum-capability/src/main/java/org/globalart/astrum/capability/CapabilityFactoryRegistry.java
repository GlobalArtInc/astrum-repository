
package org.globalart.astrum.capability;

/**
 * Registry of capability factories.
 *
 * @since capabilities 2.0
 */
public interface CapabilityFactoryRegistry
{

  /**
   * Registers a factory.
   *
   * @param type type of capabilities created by factory
   * @param factory to be added
   * @return itself, for fluent api usage
   * @throws IllegalArgumentException if another factory for same type was already registered
   */
  CapabilityFactoryRegistry register(CapabilityType type, CapabilityFactory factory);

  /**
   * Unregisters factory with specified type. If a factory with specified type was not registered before it returns
   * silently.
   *
   * @param type of factory to be removed
   * @return itself, for fluent api usage
   */
  CapabilityFactoryRegistry unregister(CapabilityType type);

  /**
   * Returns the factory bounded to specified type.
   *
   * @param type of factory
   * @return bounded factory or null if none was bounded to specified type
   */
  CapabilityFactory get(CapabilityType type);

}
