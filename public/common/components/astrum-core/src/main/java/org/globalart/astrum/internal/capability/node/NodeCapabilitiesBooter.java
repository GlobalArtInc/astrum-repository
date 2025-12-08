
package org.globalart.astrum.internal.capability.node;

import java.util.Collections;

import org.globalart.astrum.capability.CapabilityBooterSupport;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.springframework.stereotype.Component;

/**
 * Node capability booter.
 *
 * @since 3.0
 */
@Component
public class NodeCapabilitiesBooter
    extends CapabilityBooterSupport
{
  @Override
  protected void boot(final CapabilityRegistry registry) throws Exception {
    maybeAddCapability(
        registry,
        IdentityCapabilityDescriptor.TYPE,
        true, // enabled
        null, // no notes
        Collections.<String, String>emptyMap());
  }
}
