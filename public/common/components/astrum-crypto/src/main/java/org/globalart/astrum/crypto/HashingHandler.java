
package org.globalart.astrum.crypto;

import java.security.spec.InvalidKeySpecException;

import org.globalart.astrum.crypto.internal.error.CipherException;
import org.globalart.astrum.crypto.secrets.EncryptedSecret;

public interface HashingHandler
{
  EncryptedSecret hash(final char[] valueToHash) throws CipherException;

  boolean verify(final char[] input, final String expectedValue) throws CipherException, InvalidKeySpecException;
}
