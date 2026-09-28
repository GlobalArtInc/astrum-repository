
package com.globalart.astrum.ssl.plugin.internal.keystore;

import java.util.List;
import java.util.Optional;

import org.globalart.astrum.datastore.api.DataAccess;

/**
 * {@link TrustedSSLCertificateDAO} access.
 *
 */
public interface TrustedSSLCertificateDAO
    extends DataAccess
{
  Optional<TrustedSSLCertificateData> find(String alias);

  List<TrustedSSLCertificateData> findAll();

  boolean save(TrustedSSLCertificateData entity);

  boolean delete(String alias);
}
