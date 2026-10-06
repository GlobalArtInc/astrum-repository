
package org.globalart.astrum.self.hosted.internal.capability.proxy;

import java.util.Map;

import org.globalart.astrum.capability.CapabilityConfigurationSupport;

/**
 * Configuration for ForwardedRequestCapability.
 */
public class ForwardedRequestCapabilityConfiguration
    extends CapabilityConfigurationSupport
{
  private static final String ENABLED = "enabled";

  private final boolean enabled;

  public ForwardedRequestCapabilityConfiguration(final Map<String, String> properties) {
    this.enabled = parseBoolean(properties.get(ENABLED), true);
  }

  public boolean isEnabled() {
    return enabled;
  }
}
