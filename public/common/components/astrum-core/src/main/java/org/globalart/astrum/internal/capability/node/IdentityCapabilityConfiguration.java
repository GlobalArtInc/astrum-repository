
package org.globalart.astrum.internal.capability.node;

import java.util.Map;

import org.globalart.astrum.capability.CapabilityConfigurationSupport;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Configuration adapter for {@code [Orient]IdentityCapability}.
 *
 * @since 3.0
 */
public class IdentityCapabilityConfiguration
    extends CapabilityConfigurationSupport
{
  public IdentityCapabilityConfiguration(final Map<String, String> properties) {
    checkNotNull(properties);
    // No specific configuration
  }
}
