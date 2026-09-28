
package com.globalart.astrum.ssl.plugin.internal.keystore;

import java.io.IOException;
import java.security.KeyStore;
import java.security.KeyStoreException;
import java.security.NoSuchAlgorithmException;
import java.security.cert.Certificate;
import java.security.cert.CertificateException;
import java.util.List;
import org.springframework.beans.factory.annotation.Autowired;
import javax.net.ssl.TrustManager;
import javax.net.ssl.TrustManagerFactory;

import org.globalart.astrum.crypto.CryptoHelper;
import org.globalart.astrum.ssl.CertificateUtil;
import org.globalart.astrum.ssl.KeyStoreManagerConfiguration;
import org.globalart.astrum.ssl.KeystoreException;
import org.globalart.astrum.ssl.internal.ReloadableX509TrustManager;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * key-store manager for only trusted certificates.
 *
 */
@Component
public class TrustedKeyStoreManager
{
  private final CryptoHelper crypto;

  private final KeyStoreManagerConfiguration config;

  @Autowired
  public TrustedKeyStoreManager(
      final CryptoHelper crypto,
      @Qualifier("ssl") KeyStoreManagerConfiguration config)
  {
    this.crypto = checkNotNull(crypto);
    this.config = checkNotNull(config);
  }

  public TrustManager[] getTrustManagers(final List<String> certificatesAsPem) {
    try {
      KeyStore keyStore = crypto.createKeyStore(config.getKeyStoreType());
      keyStore.load(null, config.getTrustedKeyStorePassword());

      for (String pem : certificatesAsPem) {
        Certificate certificate = CertificateUtil.decodePEMFormattedCertificate(pem);
        String fingerprintAlias = CertificateUtil.calculateFingerprint(certificate);
        keyStore.setCertificateEntry(fingerprintAlias, certificate);
      }

      TrustManagerFactory trustFactory = crypto.createTrustManagerFactory(config.getTrustManagerAlgorithm());
      trustFactory.init(keyStore);
      TrustManager[] trustManagers = trustFactory.getTrustManagers();

      // replace the default X509TrustManager with our ReloadableX509TrustManager into trustManagers
      ReloadableX509TrustManager.replaceX509TrustManager(null, trustManagers);

      return trustManagers;
    }
    catch (KeyStoreException | IOException | NoSuchAlgorithmException | CertificateException e) {
      throw new RuntimeException(e);
    }
  }

  /**
   * Load a certificate into the keystore. This a way to validate the certificate does not have issues
   *
   * @param fingerprintAlias The alias to use for the certificate.
   * @param certificate The certificate to load.
   * @throws KeystoreException if the certificate cannot be loaded into the keystore.
   */
  public void validateCertificateIntoKeyStore(
      final String fingerprintAlias,
      final Certificate certificate) throws KeystoreException
  {
    try {
      KeyStore keyStore = crypto.createKeyStore(config.getKeyStoreType());
      keyStore.load(null, config.getTrustedKeyStorePassword());
      keyStore.setCertificateEntry(fingerprintAlias, certificate);
    }
    catch (CertificateException | KeyStoreException | IOException | NoSuchAlgorithmException e) {
      throw new KeystoreException("The certificate cannot be loaded into the keystore", e);
    }
  }
}
