
package com.globalart.astrum.ssl.plugin.internal.ui;

/**
 * Certificate exchange object.
 *
 * @since 3.0
 */
public record CertificateXO(
    String id,
    String fingerprint,
    String pem,
    String serialNumber,
    String subjectCommonName,
    String subjectOrganization,
    String subjectOrganizationalUnit,
    String issuerCommonName,
    String issuerOrganization,
    String issuerOrganizationalUnit,
    long issuedOn,
    long expiresOn,
    boolean inTrustStore)
{
  public CertificateXO(final String id, final String fingerprint, final String pem) {
    this(id, fingerprint, pem, null, null, null, null, null, null, null, 0, 0, false);
  }
}
