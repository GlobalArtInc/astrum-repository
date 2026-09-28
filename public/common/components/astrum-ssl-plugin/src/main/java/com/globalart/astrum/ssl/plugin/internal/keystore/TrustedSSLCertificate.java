
package com.globalart.astrum.ssl.plugin.internal.keystore;

public interface TrustedSSLCertificate
{
  /**
   * Return the alias of the SSL certificate
   */
  String getAlias();

  /**
   * Return the PEM format of the SSL certificate
   */
  String getPem();
}
