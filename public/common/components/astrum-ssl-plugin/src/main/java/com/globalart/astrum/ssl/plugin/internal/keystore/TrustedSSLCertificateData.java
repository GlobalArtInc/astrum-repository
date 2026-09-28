
package com.globalart.astrum.ssl.plugin.internal.keystore;

/**
 * {@link TrustedSSLCertificate} data.
 *
 */
public class TrustedSSLCertificateData
    implements TrustedSSLCertificate
{
  private String alias;

  private String pem;

  public TrustedSSLCertificateData() {
  }

  public TrustedSSLCertificateData(final String alias, final String pem) {
    this.alias = alias;
    this.pem = pem;
  }

  @Override
  public String getAlias() {
    return alias;
  }

  public void setAlias(final String alias) {
    this.alias = alias;
  }

  @Override
  public String getPem() {
    return pem;
  }

  public void setPem(final String pem) {
    this.pem = pem;
  }
}
