
package org.globalart.astrum.self.hosted.internal.capability.proxy;

import org.globalart.astrum.capability.CapabilityBooterSupport;
import org.globalart.astrum.capability.CapabilityRegistry;

import org.springframework.stereotype.Component;

/**
 * Boots the {@link ForwardedRequestCapability}.
 *
 * <p>
 * This ensures the capability is automatically created on first startup
 * with default settings (enabled).
 * </p>
 */
@Component
public class ForwardedRequestCapabilityBooter
    extends CapabilityBooterSupport
{
  public ForwardedRequestCapabilityBooter() {
    super(ForwardedRequestCapabilityUpgradeStep_2_131.VERSION);
  }

  @Override
  protected void boot(final CapabilityRegistry registry) throws Exception {
    // Create the capability in enabled state by default
    // This ensures backwards compatibility - forwarded headers are processed by default
    maybeAddCapability(registry, ForwardedRequestCapability.TYPE, true, null, null);
  }
}
