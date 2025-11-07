
package org.globalart.astrum.crypto.apikey;

import org.globalart.astrum.crypto.secrets.MissingKeyException;
import org.globalart.astrum.crypto.secrets.ReEncryptionNotSupportedException;

public interface ApiKeysReEncryptService
{
  String submitReEncryption(
      String algorithmForDecryption,
      Integer iterationsForDecryption,
      String notifyEmail) throws MissingKeyException, ReEncryptionNotSupportedException;
}
