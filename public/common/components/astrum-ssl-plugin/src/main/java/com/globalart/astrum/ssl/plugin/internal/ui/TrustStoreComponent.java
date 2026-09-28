
package com.globalart.astrum.ssl.plugin.internal.ui;

import java.security.cert.Certificate;
import java.security.cert.X509Certificate;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotEmpty;

import com.globalart.astrum.ssl.plugin.validator.PemCertificate;

import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.ssl.CertificateUtil;
import org.globalart.astrum.ssl.KeystoreException;
import org.globalart.astrum.ssl.TrustStore;
import org.globalart.astrum.validation.Validate;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.springframework.context.annotation.Lazy;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.ssl.CertificateUtil.calculateFingerprint;
import org.springframework.stereotype.Component;

/**
 * SSL TrustStore {@link DirectComponent}.
 *
 * @since 3.0
 */
@Component
@DirectAction(action = "ssl_TrustStore")
public class TrustStoreComponent
    extends DirectComponentSupport
{
  private final TrustStore trustStore;

  @Autowired
  public TrustStoreComponent(@Lazy final TrustStore trustStore) {
    this.trustStore = checkNotNull(trustStore);
  }

  /**
   * Retrieves certificates.
   *
   * @return a list of certificates
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:ssl-truststore:read")
  public List<CertificateXO> read() throws Exception {
    List<CertificateXO> list = new ArrayList<>();
    for (Certificate certificate : trustStore.getTrustedCertificates()) {
      list.add(asCertificateXO(certificate, true));
    }
    return list;
  }

  /**
   * Creates a certificate.
   *
   * @param pem certificate in PEM format
   * @return created certificate
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:ssl-truststore:create")
  @Validate
  public CertificateXO create(final @NotBlank @PemCertificate String pem) throws Exception {
    Certificate certificate = CertificateUtil.decodePEMFormattedCertificate(pem);
    trustStore.importTrustCertificate(certificate, calculateFingerprint(certificate));
    return asCertificateXO(certificate, true);
  }

  /**
   * Deletes a certificate.
   *
   * @param id of certificate to be deleted
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:ssl-truststore:delete")
  @Validate
  public void remove(final @NotEmpty String id) throws KeystoreException {
    trustStore.removeTrustCertificate(id);
  }

  static CertificateXO asCertificateXO(final Certificate certificate, final boolean inTrustStore) throws Exception {
    String fingerprint = calculateFingerprint(certificate);

    if (certificate instanceof X509Certificate) {
      X509Certificate x509Certificate = (X509Certificate) certificate;

      Map<String, String> subjectRdns = CertificateUtil.getSubjectRdns(x509Certificate);
      Map<String, String> issuerRdns = CertificateUtil.getIssuerRdns(x509Certificate);

      return new CertificateXO(fingerprint, fingerprint, CertificateUtil.serializeCertificateInPEM(certificate),
          x509Certificate.getSerialNumber().toString(), subjectRdns.get("CN"), subjectRdns.get("O"),
          subjectRdns.get("OU"), issuerRdns.get("CN"), issuerRdns.get("O"), issuerRdns.get("OU"),
          x509Certificate.getNotBefore().getTime(), x509Certificate.getNotAfter().getTime(), inTrustStore);
    }
    return new CertificateXO(fingerprint, fingerprint, CertificateUtil.serializeCertificateInPEM(certificate));
  }
}
