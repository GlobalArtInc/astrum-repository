
package org.globalart.astrum.email;

import org.globalart.astrum.crypto.secrets.Secret;

/**
 * Email configuration.
 *
 * @since 3.0
 */
public interface EmailConfiguration
{
  boolean isEnabled();

  void setEnabled(boolean enabled);

  String getHost();

  void setHost(String host);

  int getPort();

  void setPort(int port);

  String getUsername();

  void setUsername(String username);

  Secret getPassword();

  void setPassword(Secret password);

  String getFromAddress();

  void setFromAddress(String fromAddress);

  String getSubjectPrefix();

  void setSubjectPrefix(String subjectPrefix);

  boolean isStartTlsEnabled();

  void setStartTlsEnabled(boolean startTlsEnabled);

  boolean isStartTlsRequired();

  void setStartTlsRequired(boolean startTlsRequired);

  boolean isSslOnConnectEnabled();

  void setSslOnConnectEnabled(boolean sslOnConnectEnabled);

  boolean isSslCheckServerIdentityEnabled();

  void setSslCheckServerIdentityEnabled(boolean sslCheckServerIdentityEnabled);

  boolean isAstrumTrustStoreEnabled();

  void setAstrumTrustStoreEnabled(boolean astrumTrustStoreEnabled);

  EmailConfiguration copy();
}
