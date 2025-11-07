
package org.globalart.astrum.crypto.internal;

import org.globalart.astrum.crypto.HashingHandler;
import org.globalart.astrum.crypto.internal.error.CipherException;

/**
 * Factory for {@link HashingHandler}s, for secrets encryption stored in the PHC string format.
 * <p>
 * for further info check
 * <a href="https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md">PHC String format</a>
 */
public interface HashingHandlerFactory
{
  HashingHandler create(String encryptedSecret) throws CipherException;

  HashingHandler create(String algorithmIdentifier, byte[] salt) throws CipherException;

  HashingHandler create(String algorithmIdentifier, byte[] salt, Integer iterations) throws CipherException;
}
