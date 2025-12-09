
package org.globalart.astrum.internal.email;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.email.EmailConfiguration;

/**
 * {@link EmailConfiguration} data.
 *
 * @since 3.21
 */
public class EmailConfigurationData
    implements Cloneable, EmailConfiguration
{
  private boolean enabled;

  private String host;

  private int port;

  private String username;

  private Secret password;

  private String fromAddress;

  private String subjectPrefix;

  private boolean startTlsEnabled;

  private boolean startTlsRequired;

  private boolean sslOnConnectEnabled;

  private boolean sslCheckServerIdentityEnabled;

  private boolean astrumTrustStoreEnabled;

  @Override
  public boolean isEnabled() {
    return enabled;
  }

  @Override
  public void setEnabled(final boolean enabled) {
    this.enabled = enabled;
  }

  @Override
  public String getHost() {
    return host;
  }

  @Override
  public void setHost(final String host) {
    this.host = host;
  }

  @Override
  public int getPort() {
    return port;
  }

  @Override
  public void setPort(final int port) {
    this.port = port;
  }

  @Override
  public String getUsername() {
    return username;
  }

  @Override
  public void setUsername(final String username) {
    this.username = username;
  }

  @Override
  public Secret getPassword() {
    return password;
  }

  @Override
  public void setPassword(final Secret password) {
    this.password = password;
  }

  @Override
  public String getFromAddress() {
    return fromAddress;
  }

  @Override
  public void setFromAddress(final String fromAddress) {
    this.fromAddress = fromAddress;
  }

  @Override
  public String getSubjectPrefix() {
    return subjectPrefix;
  }

  @Override
  public void setSubjectPrefix(final String subjectPrefix) {
    this.subjectPrefix = subjectPrefix;
  }

  @Override
  public boolean isStartTlsEnabled() {
    return startTlsEnabled;
  }

  @Override
  public void setStartTlsEnabled(final boolean startTlsEnabled) {
    this.startTlsEnabled = startTlsEnabled;
  }

  @Override
  public boolean isStartTlsRequired() {
    return startTlsRequired;
  }

  @Override
  public void setStartTlsRequired(final boolean startTlsRequired) {
    this.startTlsRequired = startTlsRequired;
  }

  @Override
  public boolean isSslOnConnectEnabled() {
    return sslOnConnectEnabled;
  }

  @Override
  public void setSslOnConnectEnabled(final boolean sslOnConnectEnabled) {
    this.sslOnConnectEnabled = sslOnConnectEnabled;
  }

  @Override
  public boolean isSslCheckServerIdentityEnabled() {
    return sslCheckServerIdentityEnabled;
  }

  @Override
  public void setSslCheckServerIdentityEnabled(final boolean sslCheckServerIdentityEnabled) {
    this.sslCheckServerIdentityEnabled = sslCheckServerIdentityEnabled;
  }

  @Override
  public boolean isAstrumTrustStoreEnabled() {
    return astrumTrustStoreEnabled;
  }

  @Override
  public void setAstrumTrustStoreEnabled(final boolean astrumTrustStoreEnabled) {
    this.astrumTrustStoreEnabled = astrumTrustStoreEnabled;
  }

  @Override
  public EmailConfigurationData copy() {
    try {
      return (EmailConfigurationData) clone();
    }
    catch (CloneNotSupportedException e) {
      throw new RuntimeException(e);
    }
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "enabled=" + enabled +
        ", host='" + host + '\'' +
        ", port=" + port +
        ", username='" + username + '\'' +
        ", password='" + Strings2.MASK + '\'' +
        ", fromAddress='" + fromAddress + '\'' +
        ", subjectPrefix='" + subjectPrefix + '\'' +
        ", startTlsEnabled=" + startTlsEnabled +
        ", startTlsRequired=" + startTlsRequired +
        ", sslOnConnectEnabled=" + sslOnConnectEnabled +
        ", sslCheckServerIdentityEnabled=" + sslCheckServerIdentityEnabled +
        ", astrumTrustStoreEnabled=" + astrumTrustStoreEnabled +
        '}';
  }
}
