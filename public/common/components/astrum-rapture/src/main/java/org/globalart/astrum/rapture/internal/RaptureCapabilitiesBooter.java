
package org.globalart.astrum.rapture.internal;

import org.globalart.astrum.capability.CapabilityBooterSupport;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.rapture.internal.settings.SettingsCapabilityConfiguration;
import org.globalart.astrum.rapture.internal.settings.SettingsCapabilityDescriptor;
import org.springframework.stereotype.Component;

/**
 * Automatically create Rapture capabilities.
 *
 * @since 3.0
 */
@Component
public class RaptureCapabilitiesBooter
    extends CapabilityBooterSupport
{
  @Override
  protected void boot(final CapabilityRegistry registry) throws Exception {
    maybeAddCapability(
        registry,
        SettingsCapabilityDescriptor.TYPE,
        true, // enabled
        null, // no notes
        new SettingsCapabilityConfiguration().asMap());
  }
}
