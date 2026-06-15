
package org.globalart.astrum.coreui;

import jakarta.validation.constraints.Email;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.validation.constraint.Hostname;
import org.globalart.astrum.validation.constraint.PortNumber;

public class EmailConfigurationXO
{
  private boolean enabled;

  @Hostname
  @NotBlank
  private String host;

  @PortNumber
  @NotNull
  private int port;

  private String username;

  private String password;

  @Email
  @NotBlank
  private String fromAddress;

  private String subjectPrefix;

  private boolean startTlsEnabled;

  private boolean startTlsRequired;

  private boolean sslOnConnectEnabled;

  private boolean sslCheckServerIdentityEnabled;

  private boolean astrumTrustStoreEnabled;

  public EmailConfigurationXO(
      boolean enabled,
      String host,
      int port,
      String username,
      String password,
      String fromAddress,
      String subjectPrefix,
      boolean startTlsEnabled,
      boolean startTlsRequired,
      boolean sslOnConnectEnabled,
      boolean sslCheckServerIdentityEnabled,
      boolean astrumTrustStoreEnabled)
  {
    this.enabled = enabled;
    this.host = host;
    this.port = port;
    this.username = username;
    this.password = password;
    this.fromAddress = fromAddress;
    this.subjectPrefix = subjectPrefix;
    this.startTlsEnabled = startTlsEnabled;
    this.startTlsRequired = startTlsRequired;
    this.sslOnConnectEnabled = sslOnConnectEnabled;
    this.sslCheckServerIdentityEnabled = sslCheckServerIdentityEnabled;
    this.astrumTrustStoreEnabled = astrumTrustStoreEnabled;
  }

  public String getHost() {
    return host;
  }

  public void setHost(final String host) {
    this.host = host;
  }

  public boolean isEnabled() {
    return enabled;
  }

  public void setEnabled(final boolean enabled) {
    this.enabled = enabled;
  }

  public int getPort() {
    return port;
  }

  public void setPort(final int port) {
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

  public boolean isSslCheckServerIdentityEnabled() {
    return sslCheckServerIdentityEnabled;
  }

  public void setSslCheckServerIdentityEnabled(final boolean sslCheckServerIdentityEnabled) {
    this.sslCheckServerIdentityEnabled = sslCheckServerIdentityEnabled;
  }

  public boolean isAstrumTrustStoreEnabled() {
    return astrumTrustStoreEnabled;
  }

  public void setAstrumTrustStoreEnabled(final boolean astrumTrustStoreEnabled) {
    this.astrumTrustStoreEnabled = astrumTrustStoreEnabled;
  }
}
