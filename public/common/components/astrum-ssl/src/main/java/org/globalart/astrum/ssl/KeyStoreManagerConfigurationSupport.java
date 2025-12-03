
package org.globalart.astrum.ssl;

import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.TrustManagerFactory;

import org.globalart.astrum.common.time.Time;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Mutable {@link KeyStoreManagerConfiguration}.
 *
 * @since 3.0
 */
public class KeyStoreManagerConfigurationSupport
    implements KeyStoreManagerConfiguration
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String DEFAULT = "DEFAULT";

  private String keyStoreType = "JKS";

  private String keyAlgorithm = "RSA";

  private int keyAlgorithmSize = 2048;

  private Time certificateValidity = Time.days(36500);

  private String signatureAlgorithm = "SHA1WITHRSA";

  private String keyManagerAlgorithm = KeyManagerFactory.getDefaultAlgorithm();

  private String trustManagerAlgorithm = TrustManagerFactory.getDefaultAlgorithm();

  private char[] privateKeyStorePassword;

  private char[] trustedKeyStorePassword;

  private char[] privateKeyPassword;

  private boolean fipsEnabled;

  public void setKeyStoreType(final String keyStoreType) {
    this.keyStoreType = checkNotNull(keyStoreType);
    log.debug("Key-store type: {}", keyStoreType);
  }

  @Override
  public String getKeyStoreType() {
    return keyStoreType;
  }

  public void setKeyAlgorithm(final String keyAlgorithm) {
    this.keyAlgorithm = checkNotNull(keyAlgorithm);
    log.debug("Key algorithm: {}", keyAlgorithm);
  }

  @Override
  public String getKeyAlgorithm() {
    return keyAlgorithm;
  }

  public void setKeyAlgorithmSize(final int keyAlgorithmSize) {
    this.keyAlgorithmSize = keyAlgorithmSize;
    log.debug("Key algorithm size: {}", keyAlgorithmSize);
  }

  @Override
  public int getKeyAlgorithmSize() {
    return keyAlgorithmSize;
  }

  public void setCertificateValidity(final Time certificateValidity) {
    this.certificateValidity = certificateValidity;
    log.debug("Certificate validity: {}", certificateValidity);
  }

  @Override
  public Time getCertificateValidity() {
    return certificateValidity;
  }

  public void setSignatureAlgorithm(final String signatureAlgorithm) {
    this.signatureAlgorithm = checkNotNull(signatureAlgorithm);
    log.debug("Signature algorithm: {}", signatureAlgorithm);
  }

  @Override
  public String getSignatureAlgorithm() {
    return signatureAlgorithm;
  }

  public void setKeyManagerAlgorithm(String keyManagerAlgorithm) {
    checkNotNull(keyManagerAlgorithm);
    if (keyManagerAlgorithm.equals(DEFAULT)) {
      this.keyManagerAlgorithm = KeyManagerFactory.getDefaultAlgorithm();
    }
    else {
      this.keyManagerAlgorithm = keyManagerAlgorithm;
    }
    log.debug("Key-manager algorithm: {}", this.keyManagerAlgorithm);
  }

  @Override
  public String getKeyManagerAlgorithm() {
    return keyManagerAlgorithm;
  }

  public void setTrustManagerAlgorithm(final String trustManagerAlgorithm) {
    checkNotNull(trustManagerAlgorithm);
    if (trustManagerAlgorithm.equals(DEFAULT)) {
      this.trustManagerAlgorithm = TrustManagerFactory.getDefaultAlgorithm();
    }
    else {
      this.trustManagerAlgorithm = trustManagerAlgorithm;
    }
    log.debug("Trust-manager algorithm: {}", this.trustManagerAlgorithm);
  }

  @Override
  public String getTrustManagerAlgorithm() {
    return trustManagerAlgorithm;
  }

  public void setPrivateKeyStorePassword(final char[] password) {
    this.privateKeyStorePassword = password;
  }

  @Override
  public char[] getPrivateKeyStorePassword() {
    return privateKeyStorePassword;
  }

  public void setTrustedKeyStorePassword(final char[] password) {
    this.trustedKeyStorePassword = password;
  }

  @Override
  public char[] getTrustedKeyStorePassword() {
    return trustedKeyStorePassword;
  }

  public void setPrivateKeyPassword(final char[] password) {
    this.privateKeyPassword = password;
  }

  @Override
  public char[] getPrivateKeyPassword() {
    return privateKeyPassword;
  }

  @Override
  public boolean isFipsEnabled() {
    return fipsEnabled;
  }

  public void setFipsEnabled(final boolean fipsEnabled) {
    this.fipsEnabled = fipsEnabled;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "keyStoreType='" + keyStoreType + '\'' +
        ", keyAlgorithm='" + keyAlgorithm + '\'' +
        ", keyAlgorithmSize=" + keyAlgorithmSize +
        ", certificateValidity=" + certificateValidity +
        ", signatureAlgorithm='" + signatureAlgorithm + '\'' +
        ", keyManagerAlgorithm='" + keyManagerAlgorithm + '\'' +
        ", trustManagerAlgorithm='" + trustManagerAlgorithm + '\'' +
        ", fipsEnabled=" + fipsEnabled +
        '}';
  }
}
