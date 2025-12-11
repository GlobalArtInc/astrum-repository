
package org.globalart.astrum.internal.ssrf;

import java.util.Collection;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.validation.ssrf.SsrfProtectionConfiguration;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Serializable data class for storing {@link SsrfProtectionConfiguration} in the key-value store.
 */
class SsrfProtectionConfigData
{
  private final boolean enabled;

  private final Set<String> allowedIPs;

  private final Set<String> allowedDomains;

  @JsonCreator
  SsrfProtectionConfigData(
      @JsonProperty("enabled") final boolean enabled,
      @JsonProperty("allowedIPs") final Set<String> allowedIPs,
      @JsonProperty("allowedDomains") final Set<String> allowedDomains)
  {
    this.enabled = enabled;
    this.allowedIPs = allowedIPs != null ? allowedIPs : Set.of();
    this.allowedDomains = allowedDomains != null ? allowedDomains : Set.of();
  }

  public boolean isEnabled() {
    return enabled;
  }

  public Set<String> getAllowedIPs() {
    return allowedIPs;
  }

  public Set<String> getAllowedDomains() {
    return allowedDomains;
  }

  SsrfProtectionConfiguration toConfiguration() {
    return new SsrfProtectionConfiguration(enabled, allowedIPs, allowedDomains);
  }

  static SsrfProtectionConfigData from(final SsrfProtectionConfiguration config) {
    checkNotNull(config);
    return new SsrfProtectionConfigData(config.enabled(), config.allowedIPs(), config.allowedDomains());
  }

  @SuppressWarnings("unchecked")
  static SsrfProtectionConfigData fromMap(final Map<String, Object> map) {
    boolean enabled = Boolean.TRUE.equals(map.get("enabled"));
    Set<String> ips = toStringSet(map.get("allowedIPs"));
    Set<String> domains = toStringSet(map.get("allowedDomains"));
    return new SsrfProtectionConfigData(enabled, ips, domains);
  }

  private static Set<String> toStringSet(final Object value) {
    if (value instanceof Collection<?> collection) {
      Set<String> result = new HashSet<>();
      for (Object item : collection) {
        if (item instanceof String s) {
          result.add(s);
        }
      }
      return result;
    }
    return Set.of();
  }
}
