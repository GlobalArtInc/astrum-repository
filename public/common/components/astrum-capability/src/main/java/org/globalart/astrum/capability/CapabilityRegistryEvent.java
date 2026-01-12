
package org.globalart.astrum.capability;

/**
 * {@link CapabilityRegistry} related events.
 *
 * @since capabilities 2.0
 */
public class CapabilityRegistryEvent
{
  private final CapabilityRegistry capabilityRegistry;

  public CapabilityRegistryEvent(final CapabilityRegistry capabilityRegistry) {
    this.capabilityRegistry = capabilityRegistry;
  }

  @Override
  public String toString() {
    return capabilityRegistry.toString();
  }

  /**
   * Event fired after capabilities were loaded loaded from persistence store.
   *
   * @since capabilities 2.0
   */
  public static class AfterLoad
      extends CapabilityRegistryEvent
  {
    public AfterLoad(final CapabilityRegistry capabilityRegistry) {
      super(capabilityRegistry);
    }

    @Override
    public String toString() {
      return "Loaded " + super.toString();
    }
  }

  /**
   * Event fired once the registry is ready on boot.
   *
   * @since 3.0
   */
  public static class Ready
      extends CapabilityRegistryEvent
  {
    private final CapabilityRegistry capabilityRegistry;

    public Ready(final CapabilityRegistry capabilityRegistry) {
      super(capabilityRegistry);
      this.capabilityRegistry = capabilityRegistry;
    }

    public CapabilityRegistry getCapabilityRegistry() {
      return capabilityRegistry;
    }

    @Override
    public String toString() {
      return "Ready " + super.toString();
    }
  }
}
