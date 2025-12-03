
package org.globalart.astrum.ssl;

import org.globalart.astrum.common.time.Time;

/**
 * {@link KeyStoreManager} configuration.
 *
 * @since 3.0
 */
public interface KeyStoreManagerConfiguration
{
  String getKeyStoreType();

  String getKeyAlgorithm();

  int getKeyAlgorithmSize();

  Time getCertificateValidity();

  String getSignatureAlgorithm();

  String getKeyManagerAlgorithm();

  String getTrustManagerAlgorithm();

  char[] getPrivateKeyStorePassword();

  char[] getTrustedKeyStorePassword();

  char[] getPrivateKeyPassword();

  boolean isFipsEnabled();
}
