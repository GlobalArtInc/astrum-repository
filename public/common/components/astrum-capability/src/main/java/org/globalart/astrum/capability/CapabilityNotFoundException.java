
package org.globalart.astrum.capability;

/**
 * Exception thrown when a capability was not found.
 *
 * @since capabilities 2.2
 */
public class CapabilityNotFoundException
    extends RuntimeException
{
  public CapabilityNotFoundException(final CapabilityIdentity id) {
    super(String.format("Capability with id '%s' was not found", id));
  }
}
