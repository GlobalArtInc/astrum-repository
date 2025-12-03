
package org.globalart.astrum.ssl;

import java.security.cert.Certificate;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link Certificate} event.
 *
 * @since 3.1
 */
public abstract class CertificateEvent
{
  private final String alias;

  private final Certificate certificate;

  public CertificateEvent(final String alias, final Certificate certificate) {
    this.alias = checkNotNull(alias);
    this.certificate = checkNotNull(certificate);
  }

  public String getAlias() {
    return alias;
  }

  public Certificate getCertificate() {
    return certificate;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "alias='" + alias + '\'' +
        ", certificate=" + certificate +
        '}';
  }
}
