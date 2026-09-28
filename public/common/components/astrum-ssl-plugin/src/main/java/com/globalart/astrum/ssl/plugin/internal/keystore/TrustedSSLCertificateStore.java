
package com.globalart.astrum.ssl.plugin.internal.keystore;

import java.util.List;
import java.util.Optional;

public interface TrustedSSLCertificateStore
{
  /**
   * Gets the trusted SSL certificate associated with the given alias.
   *
   * @return {@code null} if no alias has been associated
   */
  Optional<TrustedSSLCertificate> find(String alias);

  List<TrustedSSLCertificate> findAll();

  /**
   * Save a trusted SSL certificate.
   */
  void save(String alias, String pem);

  /**
   * Delete a trusted SSL certificate.
   */
  void delete(String alias);
}
