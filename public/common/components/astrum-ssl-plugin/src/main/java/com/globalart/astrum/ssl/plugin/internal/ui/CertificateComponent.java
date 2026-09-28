
package com.globalart.astrum.ssl.plugin.internal.ui;

import java.io.IOException;
import java.net.UnknownHostException;
import java.security.cert.Certificate;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotEmpty;

import com.globalart.astrum.ssl.plugin.validator.HostnameOrIpAddress;
import com.globalart.astrum.ssl.plugin.validator.PemCertificate;

import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.ssl.CertificateRetriever;
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
import static com.globalart.astrum.ssl.plugin.internal.ui.TrustStoreComponent.asCertificateXO;
import static org.globalart.astrum.ssl.CertificateUtil.calculateFingerprint;
import static org.globalart.astrum.ssl.CertificateUtil.decodePEMFormattedCertificate;
import org.springframework.stereotype.Component;

/**
 * SSL Certificate {@link DirectComponent}.
 */
@Component
@DirectAction(action = "ssl_Certificate")
public class CertificateComponent
    extends DirectComponentSupport
{
  private final TrustStore trustStore;

  private final CertificateRetriever certificateRetriever;

  @Autowired
  public CertificateComponent(@Lazy final TrustStore trustStore, final CertificateRetriever certificateRetriever) {
    this.trustStore = checkNotNull(trustStore);
    this.certificateRetriever = checkNotNull(certificateRetriever);
  }

  /**
   * Retrieves certificate given a host/port.
   *
   * @param host to get certificate from
   * @param port the port
   * @param protocolHint the protocol hint
   * @return certificate
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @Validate
  @RequiresAuthentication
  @RequiresPermissions("astrum:ssl-truststore:read")
  public CertificateXO retrieveFromHost(
      final @HostnameOrIpAddress @NotEmpty String host,
      final @Nullable Integer port,
      final @Nullable String protocolHint) throws Exception
  {
    Certificate[] chain;
    try {
      chain = certificateRetriever.retrieveCertificates(host, port, protocolHint);
    }
    catch (Exception e) {
      String errorMessage = e.getMessage();
      if (e instanceof UnknownHostException) {
        errorMessage = "Unknown host " + host;
      }
      throw new IOException(errorMessage);
    }
    if (chain == null || chain.length == 0) {
      int actualPort = port == null ? 443 : port;
      throw new IOException(String.format("Could not retrieve an SSL certificate from '%s:%s'", host, actualPort));
    }
    return asCertificateXO(chain[0], isInTrustStore(chain[0]));
  }

  /**
   * Retrieves certificate given a certificate pem.
   *
   * @param pem certificate in PEM format
   * @return certificate
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @Validate
  @RequiresAuthentication
  @RequiresPermissions("astrum:ssl-truststore:read")
  public CertificateXO details(final @NotBlank @PemCertificate String pem) throws Exception {
    Certificate certificate = decodePEMFormattedCertificate(pem);
    return asCertificateXO(certificate, isInTrustStore(certificate));
  }

  boolean isInTrustStore(final Certificate certificate) {
    try {
      return trustStore.getTrustedCertificate(calculateFingerprint(certificate)) != null;
    }
    catch (Exception ignore) {
      return false;
    }
  }
}
