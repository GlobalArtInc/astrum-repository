
package org.globalart.astrum.api.rest.selfhosted.security.ssrf.model;

import java.util.Set;

import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.validation.ssrf.SsrfProtectionConfiguration;

import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * REST API exchange object for SSRF protection configuration.
 */
public class SsrfProtectionConfigurationXO
{
  @NotNull
  @JsonProperty(required = true)
  @Schema(description = "Whether SSRF protection is enabled", example = "true",
      requiredMode = Schema.RequiredMode.REQUIRED)
  private Boolean enabled;

  @Schema(description = "List of IP addresses allowed to bypass SSRF protection",
      example = "[\"10.0.0.50\", \"192.168.1.100\"]")
  private Set<String> allowedIPs = Set.of();

  @Schema(description = "List of domain names allowed to bypass SSRF protection",
      example = "[\"internal.corp.com\", \"registry.local\"]")
  private Set<String> allowedDomains = Set.of();

  @SuppressWarnings("unused")
  public SsrfProtectionConfigurationXO() {
    // for deserialization
  }

  public SsrfProtectionConfigurationXO(final SsrfProtectionConfiguration config) {
    checkNotNull(config);
    this.enabled = config.enabled();
    this.allowedIPs = config.allowedIPs();
    this.allowedDomains = config.allowedDomains();
  }

  public Boolean isEnabled() {
    return enabled;
  }

  public void setEnabled(final Boolean enabled) {
    this.enabled = enabled;
  }

  public Set<String> getAllowedIPs() {
    return allowedIPs;
  }

  public void setAllowedIPs(final Set<String> allowedIPs) {
    this.allowedIPs = allowedIPs != null ? allowedIPs : Set.of();
  }

  public Set<String> getAllowedDomains() {
    return allowedDomains;
  }

  public void setAllowedDomains(final Set<String> allowedDomains) {
    this.allowedDomains = allowedDomains != null ? allowedDomains : Set.of();
  }

  public SsrfProtectionConfiguration toConfiguration() {
    return new SsrfProtectionConfiguration(
        checkNotNull(enabled, "'enabled' is required"),
        allowedIPs,
        allowedDomains);
  }
}
