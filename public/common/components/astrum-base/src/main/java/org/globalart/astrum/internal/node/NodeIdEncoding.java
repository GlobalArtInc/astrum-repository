
package org.globalart.astrum.internal.node;

import java.security.cert.Certificate;
import java.security.cert.CertificateEncodingException;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.ssl.CertificateUtil;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Node ID encoding helpers.
 *
 * @since 3.0
 */
public class NodeIdEncoding
{
  private NodeIdEncoding() {
    // empty
  }

  /**
   * Encode plain Certificate SHA1 into node-id string.
   */
  public static String nodeIdForSha1(final String input) {
    checkNotNull(input);
    return Strings2.encodeSeparator(input, '-', 8);
  }

  /**
   * Decode node-id into plain SHA1 string.
   */
  public static String sha1ForNodeId(final String input) {
    checkNotNull(input);
    return input.replaceAll("-", "");
  }

  /**
   * Return node-id for certificate.
   */
  public static String nodeIdForCertificate(final Certificate cert) throws CertificateEncodingException {
    checkNotNull(cert);
    String sha1 = CertificateUtil.calculateSha1(cert);
    return nodeIdForSha1(sha1);
  }

  /**
   * Return node-id for certificate fingerprint.
   */
  public static String nodeIdForFingerprint(final String fingerprint) {
    checkNotNull(fingerprint);
    String sha1 = fingerprint.replace(":", "");
    return nodeIdForSha1(sha1);
  }
}
