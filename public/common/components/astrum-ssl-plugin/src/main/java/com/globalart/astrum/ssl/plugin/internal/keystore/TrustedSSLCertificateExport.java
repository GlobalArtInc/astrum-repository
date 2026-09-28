
package com.globalart.astrum.ssl.plugin.internal.keystore;

import java.io.File;
import java.io.IOException;
import java.util.List;

import org.globalart.astrum.supportzip.ExportSecurityData;
import org.globalart.astrum.supportzip.ImportData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Export/Import {@link TrustedSSLCertificate} data to/from JSON file for support zip.
 */
@Component
@Qualifier("trustedSslCertificateExport")
public class TrustedSSLCertificateExport
    extends JsonExporter
    implements ExportSecurityData, ImportData
{
  private final TrustedSSLCertificateStore trustedSSLCertificateStore;

  @Autowired
  public TrustedSSLCertificateExport(final TrustedSSLCertificateStore trustedSSLCertificateStore) {
    this.trustedSSLCertificateStore = checkNotNull(trustedSSLCertificateStore);
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export TrustedSSLCertificate to {}", file);
    List<TrustedSSLCertificate> certificateList = trustedSSLCertificateStore.findAll();
    exportToJson(certificateList, file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring TrustedSSLCertificate from {}", file);
    List<TrustedSSLCertificateData> certificateList = importFromJson(file, TrustedSSLCertificateData.class);
    for (TrustedSSLCertificateData certificate : certificateList) {
      trustedSSLCertificateStore.save(certificate.getAlias(), certificate.getPem());
    }
  }
}
