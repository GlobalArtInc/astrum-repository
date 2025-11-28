
package org.globalart.astrum.validation.ssrf;

import java.util.Set;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Configuration for SSRF protection settings, including the enable/disable toggle and allow lists.
 */
public record SsrfProtectionConfiguration(
    boolean enabled,
    Set<String> allowedIPs,
    Set<String> allowedDomains)
{
  public SsrfProtectionConfiguration {
    checkNotNull(allowedIPs);
    checkNotNull(allowedDomains);
    allowedIPs = Set.copyOf(allowedIPs);
    allowedDomains = Set.copyOf(allowedDomains);
  }
}
