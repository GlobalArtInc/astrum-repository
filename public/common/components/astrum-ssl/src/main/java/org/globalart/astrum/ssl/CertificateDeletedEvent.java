
package org.globalart.astrum.ssl;

import java.security.cert.Certificate;

/**
 * Emitted when a {@link Certificate} has been deleted.
 */
public class CertificateDeletedEvent
    extends CertificateEvent
{
  public CertificateDeletedEvent(final String alias, final Certificate certificate) {
    super(alias, certificate);
  }
}
