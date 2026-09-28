
package com.globalart.astrum.ssl.plugin.tasks;

import java.security.cert.Certificate;
import java.util.Collection;

import org.springframework.beans.factory.annotation.Autowired;
import com.globalart.astrum.ssl.plugin.internal.keystore.TrustedSSLCertificateStore;

import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.ssl.CertificateUtil;
import org.globalart.astrum.ssl.KeyStoreManager;
import org.globalart.astrum.ssl.KeystoreException;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.globalart.astrum.ssl.plugin.internal.TrustStoreImpl.TRUSTED_CERTIFICATES_MIGRATION_COMPLETE;

/**
 * Service to migrate trusted certificates from key_store_data to trusted_ssl_certificate.
 */
@Lazy
@Component
public class TrustedCertificateMigrationService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final TrustedSSLCertificateStore trustedSSLCertificateStore;

  private final KeyStoreManager keyStoreManager;

  private final GlobalKeyValueStore globalKeyValueStore;

  @Autowired
  public TrustedCertificateMigrationService(
      @Qualifier("ssl") final KeyStoreManager keyStoreManager,
      final TrustedSSLCertificateStore trustedSSLCertificateStore,
      final GlobalKeyValueStore globalKeyValueStore)
  {
    this.trustedSSLCertificateStore = checkNotNull(trustedSSLCertificateStore);
    this.keyStoreManager = checkNotNull(keyStoreManager);
    this.globalKeyValueStore = checkNotNull(globalKeyValueStore);
  }

  public void migrate() {
    long start = System.currentTimeMillis();
    Collection<Certificate> trustedCertificates;
    log.info("Starting migration.");
    try {
      trustedCertificates = keyStoreManager.getTrustedCertificates();
    }
    catch (KeystoreException e) {
      log.error("An error occurred trying to get the trusted certificates.", e);
      return;
    }

    log.info("There are {} trusted certificates to migrate.", trustedCertificates.size());

    for (Certificate certificate : trustedCertificates) {
      try {
        String alias = CertificateUtil.calculateFingerprint(certificate);
        String pem = CertificateUtil.serializeCertificateInPEM(certificate);

        // save or update
        trustedSSLCertificateStore.save(alias, pem);
        log.info("Certificate with alias {} was migrated successfully.", alias);
      }
      catch (Exception e) {
        log.error("An error occurred trying to migrate trusted certificate: {}.", certificate, e);
      }
    }

    globalKeyValueStore.setBoolean(TRUSTED_CERTIFICATES_MIGRATION_COMPLETE, true);

    long finish = System.currentTimeMillis();
    long timeElapsed = finish - start;
    log.info("The migration has been completed successfully, it took {} milliseconds.", timeElapsed);
  }
}
