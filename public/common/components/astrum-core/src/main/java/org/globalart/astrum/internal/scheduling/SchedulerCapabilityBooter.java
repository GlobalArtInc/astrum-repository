
package org.globalart.astrum.internal.scheduling;

import org.globalart.astrum.capability.CapabilityBooterSupport;
import org.globalart.astrum.capability.CapabilityRegistry;

import org.springframework.stereotype.Component;

/**
 * Creates {@link SchedulerCapability}.
 *
 * @since 3.0
 */
@Component
public class SchedulerCapabilityBooter
    extends CapabilityBooterSupport
{
  @Override
  protected void boot(final CapabilityRegistry registry) throws Exception {
    maybeAddCapability(registry, SchedulerCapabilityDescriptor.TYPE, true, null, null);
  }
}
