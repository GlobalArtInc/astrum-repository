
package org.globalart.astrum.crypto.secrets;

import javax.annotation.Nullable;

public interface ReEncryptService
{
  /**
   * Checks the provided key is available to all nodes. If the key is available, a re-encryption task is submitted.
   *
   * @param secretKeyId the key to use for re-encryption
   * @param notifyEmail the email address to notify when the re-encryption task is complete
   * @return the id of the re-encryption task
   */
  String submitReEncryption(
      String secretKeyId,
      @Nullable String notifyEmail) throws MissingKeyException, ReEncryptionNotSupportedException;
}
