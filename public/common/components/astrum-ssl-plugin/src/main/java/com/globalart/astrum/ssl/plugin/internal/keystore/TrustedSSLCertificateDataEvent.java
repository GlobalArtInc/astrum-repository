
package com.globalart.astrum.ssl.plugin.internal.keystore;

/**
 * Event sent out when {@link TrustedSSLCertificateData} data changes.
 *
 */
public interface TrustedSSLCertificateDataEvent
{
  String getAlias();
}
