
package org.globalart.astrum.capability;

import java.util.Map;

/**
 * Provides access to capability context.
 *
 * @since capabilities 2.0
 */
public interface CapabilityContext
{

  /**
   * Returns an unique capability identifier.
   *
   * @return identifier
   */
  CapabilityIdentity id();

  /**
   * Returns type of capability.
   *
   * @return capability type (never null)
   */
  CapabilityType type();

  /**
   * Returns descriptor of capability.
   *
   * @return capability descriptor (never null)
   */
  CapabilityDescriptor descriptor();

  /**
   * Returns capability notes.
   *
   * @return capability notes (can be null)
   */
  String notes();

  /**
   * Current capability properties.
   *
   * @return properties.
   */
  Map<String, String> properties();

  /**
   * Whether the capability is enabled.
   *
   * @return true, if capability is enabled
   */
  boolean isEnabled();

  /**
   * Whether the capability is active.
   *
   * @return true, if capability was activated and not yet passivated
   */
  boolean isActive();

  /**
   * Whether the capability had failed a lifecycle callback method (create/load/update/activate/passivate).
   *
   * @return true, if capability had failed a callback method
   */
  boolean hasFailure();

  /**
   * Last exception thrown during a lifecycle callback method (create/load/update/activate/passivate).
   *
   * @return last exception thrown during a lifecycle callback method or null if it not failed
   */
  Exception failure();

  /**
   * Returns name of last lifecycle callback method (create/load/update/activate/passivate) that failed.
   *
   * @since 2.7
   */
  String failingAction();

  /**
   * Describe current state.
   *
   * @return state description
   */
  String stateDescription();

}
