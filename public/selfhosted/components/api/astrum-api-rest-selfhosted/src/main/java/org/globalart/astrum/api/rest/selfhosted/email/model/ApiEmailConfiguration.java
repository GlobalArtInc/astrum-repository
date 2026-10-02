
package org.globalart.astrum.api.rest.selfhosted.email.model;

import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.validation.constraint.Hostname;
import org.globalart.astrum.validation.constraint.PortNumber;

import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.Email;
import jakarta.validation.constraints.NotBlank;

public class ApiEmailConfiguration
{
  private boolean enabled;

  @Hostname
  @NotBlank
  private String host;

  @PortNumber
  @NotNull
  private Integer port;

  private String username;

  private String password;

  @Email
  @NotBlank
  @Schema(example = "astrum@example.org")
  private String fromAddress;

  @Schema(description = "A prefix to add to all email subjects to aid in identifying automated emails")
  private String subjectPrefix;

  @Schema(description = "Enable STARTTLS Support for Insecure Connections")
  private boolean startTlsEnabled;

  @Schema(description = "Require STARTTLS Support")
  private boolean startTlsRequired;

  @Schema(description = "Enable SSL/TLS Encryption upon Connection")
  private boolean sslOnConnectEnabled;

  @Schema(description = "Verify the server certificate when using TLS or SSL")
  private boolean sslServerIdentityCheckEnabled;

  @Schema(description = "Use the Astrum Repository Manager's certificate truststore")
  private boolean astrumTrustStoreEnabled;

  public boolean isEnabled() {
    return enabled;
  }

  public void setEnabled(final boolean enabled) {
    this.enabled = enabled;
  }

  public String getHost() {
    return host;
  }

  public void setHost(final String host) {
    this.host = host;
  }

  public Integer getPort() {
    return port;
  }

  public void setPort(final Integer port) {
    this.port = port;
  }

  public String getUsername() {
    return username;
  }

  public void setUsername(final String username) {
    this.username = username;
  }

  public String getPassword() {
    return password;
  }

  public void setPassword(final String password) {
    this.password = password;
  }

  public String getFromAddress() {
    return fromAddress;
  }

  public void setFromAddress(final String fromAddress) {
    this.fromAddress = fromAddress;
  }

  public String getSubjectPrefix() {
    return subjectPrefix;
  }

  public void setSubjectPrefix(final String subjectPrefix) {
    this.subjectPrefix = subjectPrefix;
  }

  public boolean isStartTlsEnabled() {
    return startTlsEnabled;
  }

  public void setStartTlsEnabled(final boolean startTlsEnabled) {
    this.startTlsEnabled = startTlsEnabled;
  }

  public boolean isStartTlsRequired() {
    return startTlsRequired;
  }

  public void setStartTlsRequired(final boolean startTlsRequired) {
    this.startTlsRequired = startTlsRequired;
  }

  public boolean isSslOnConnectEnabled() {
    return sslOnConnectEnabled;
  }

  public void setSslOnConnectEnabled(final boolean sslOnConnectEnabled) {
    this.sslOnConnectEnabled = sslOnConnectEnabled;
  }

  public boolean isSslServerIdentityCheckEnabled() {
    return sslServerIdentityCheckEnabled;
  }

  public void setSslServerIdentityCheckEnabled(final boolean sslServerIdentityCheckEnabled) {
    this.sslServerIdentityCheckEnabled = sslServerIdentityCheckEnabled;
  }

  public boolean isAstrumTrustStoreEnabled() {
    return astrumTrustStoreEnabled;
  }

  public void setAstrumTrustStoreEnabled(final boolean astrumTrustStoreEnabled) {
    this.astrumTrustStoreEnabled = astrumTrustStoreEnabled;
  }
}
