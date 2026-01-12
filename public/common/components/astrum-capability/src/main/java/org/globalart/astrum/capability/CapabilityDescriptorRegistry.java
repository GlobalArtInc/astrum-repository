
package org.globalart.astrum.capability;

/**
 * Registry of known {@link CapabilityDescriptor} components.
 */
public interface CapabilityDescriptorRegistry
{
  CapabilityDescriptorRegistry register(CapabilityDescriptor capabilityDescriptor);

  CapabilityDescriptorRegistry unregister(CapabilityDescriptor capabilityDescriptor);

  CapabilityDescriptor get(CapabilityType capabilityType);

  CapabilityDescriptor[] getAll();
}
