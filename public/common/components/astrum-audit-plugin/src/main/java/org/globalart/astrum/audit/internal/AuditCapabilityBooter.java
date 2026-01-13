
package org.globalart.astrum.audit.internal;

import org.globalart.astrum.capability.CapabilityBooterSupport;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.springframework.stereotype.Component;

/**
 * Audit {@link CapabilityBooterSupport}.
 *
 * @since 3.1
 */
@Component
public class AuditCapabilityBooter
    extends CapabilityBooterSupport
{
  @Override
  protected void boot(final CapabilityRegistry registry) throws Exception {
    maybeAddCapability(registry, AuditCapability.TYPE, true, null, null);
  }
}
