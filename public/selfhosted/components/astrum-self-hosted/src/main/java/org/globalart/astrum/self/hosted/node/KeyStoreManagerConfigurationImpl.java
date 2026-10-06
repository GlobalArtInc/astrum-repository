
package org.globalart.astrum.self.hosted.node;

import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.common.app.FeatureFlags;
import org.globalart.astrum.ssl.KeyStoreManagerConfiguration;
import org.globalart.astrum.ssl.KeyStoreManagerConfigurationSupport;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * Node {@link KeyStoreManagerConfiguration}.
 *
 * @since 3.0
 */
@Component
@Qualifier(NodeKeyStoreManagerImpl.NAME)
public class KeyStoreManagerConfigurationImpl
    extends KeyStoreManagerConfigurationSupport
{
  private static final String CPREFIX = "${node.keyStoreManager";

  /**
   * Private key-store password.
   */
  private static final char[] PKSP = "uuPWrk3UEQRaolpd".toCharArray();

  /**
   * Trusted key-store password.
   */
  private static final char[] TKSP = "1bmcqcHV3sp6fVKD".toCharArray();

  /**
   * Private-key password.
   */
  private static final char[] PKP = "CyQM8zCFeorarTA8".toCharArray();

  @Autowired
  public KeyStoreManagerConfigurationImpl(
      @Value(CPREFIX + ".keyStoreType:JKS}") final String keyStoreType,
      @Value(CPREFIX + ".keyAlgorithm:RSA}") final String keyAlgorithm,
      @Value(CPREFIX + ".keyAlgorithmSize:2048}") final int keyAlgorithmSize,
      @Value(CPREFIX + ".certificateValidity:36500d}") final Time certificateValidity,
      @Value(CPREFIX + ".signatureAlgorithm:SHA1WITHRSA}") final String signatureAlgorithm,
      @Value(CPREFIX + ".keyManagerAlgorithm:DEFAULT}") final String keyManagerAlgorithm,
      @Value(CPREFIX + ".trustManagerAlgorithm:DEFAULT}") final String trustManagerAlgorithm,
      @Value(FeatureFlags.ASTRUM_SECURITY_FIPS_ENABLED_NAMED_VALUE) final boolean astrumSecurityFipsEnabled)
  {
    setPrivateKeyStorePassword(PKSP);
    setTrustedKeyStorePassword(TKSP);
    setPrivateKeyPassword(PKP);
    setKeyStoreType(keyStoreType);
    setKeyAlgorithm(keyAlgorithm);
    setKeyAlgorithmSize(keyAlgorithmSize);
    setCertificateValidity(certificateValidity);
    setSignatureAlgorithm(signatureAlgorithm);
    setKeyManagerAlgorithm(keyManagerAlgorithm);
    setTrustManagerAlgorithm(trustManagerAlgorithm);
    setFipsEnabled(astrumSecurityFipsEnabled);
  }

  @VisibleForTesting
  public KeyStoreManagerConfigurationImpl() {
    setPrivateKeyStorePassword(PKSP);
    setTrustedKeyStorePassword(TKSP);
    setPrivateKeyPassword(PKP);
  }
}
