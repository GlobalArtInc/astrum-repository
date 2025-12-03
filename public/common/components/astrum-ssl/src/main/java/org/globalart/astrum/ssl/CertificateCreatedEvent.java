
package org.globalart.astrum.ssl;

import java.security.cert.Certificate;

/**
 * Emitted when a {@link Certificate} has been created.
 */
public class CertificateCreatedEvent
    extends CertificateEvent
{
  public CertificateCreatedEvent(final String alias, final Certificate certificate) {
    super(alias, certificate);
  }
}
