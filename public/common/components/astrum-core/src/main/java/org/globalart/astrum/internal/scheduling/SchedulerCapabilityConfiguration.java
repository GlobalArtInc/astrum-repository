
package org.globalart.astrum.internal.scheduling;

import java.util.Map;

import org.globalart.astrum.capability.CapabilityConfigurationSupport;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link SchedulerCapability} configuration.
 *
 * @since 3.0
 */
public class SchedulerCapabilityConfiguration
    extends CapabilityConfigurationSupport
{
  public SchedulerCapabilityConfiguration(final Map<String, String> properties) {
    checkNotNull(properties);
    // No specific configuration
  }
}
