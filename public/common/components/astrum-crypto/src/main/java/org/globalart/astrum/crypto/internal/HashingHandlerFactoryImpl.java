
package org.globalart.astrum.crypto.internal;

import org.globalart.astrum.crypto.CryptoHelper;
import org.globalart.astrum.crypto.HashingHandler;
import org.globalart.astrum.crypto.internal.error.CipherException;
import org.globalart.astrum.crypto.secrets.EncryptedSecret;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.crypto.internal.EncryptionHelper.KEY_ITERATION_PHC;
import static org.globalart.astrum.crypto.internal.EncryptionHelper.fromBase64;

/**
 * Default implementation for {@link HashingHandlerFactory} . provides a simple hashing handler supporting PHC string
 * format
 */
@Component
public class HashingHandlerFactoryImpl
    implements HashingHandlerFactory
{
  public static final String KEY_FACTORY_ALGORITHM_SHA1 = "PBKDF2WithHmacSHA1";

  public static final String KEY_FACTORY_ALGORITHM_SHA256 = "PBKDF2WithHmacSHA256";

  private static final int KEY_LENGTH_SHA1 = 128;

  private static final int KEY_LENGTH_SHA256 = 256;

  public static final int DEFAULT_ITERATIONS_SHA1 = 1024;

  public static final int DEFAULT_ITERATIONS_SHA256 = 10000;

  private final CryptoHelper cryptoHelper;

  @Autowired
  public HashingHandlerFactoryImpl(final CryptoHelper cryptoHelper) {
    this.cryptoHelper = checkNotNull(cryptoHelper);
  }

  public HashingHandler create(final String encryptedSecret) throws CipherException {
    checkNotNull(encryptedSecret);
    EncryptedSecret parsedEncryptedSecret = EncryptedSecret.parse(encryptedSecret);
    String algorithm = parsedEncryptedSecret.getAlgorithm();
    byte[] salt = fromBase64(parsedEncryptedSecret.getSalt());

    // Extract iterations from the encrypted secret if present
    String iterationsStr = parsedEncryptedSecret.getAttributes().get(KEY_ITERATION_PHC);
    Integer iterations = null;
    if (iterationsStr != null) {
      try {
        iterations = Integer.parseInt(iterationsStr);
      }
      catch (NumberFormatException e) {
        // If parsing fails, iterations will remain null and default will be used
      }
    }

    return create(algorithm, salt, iterations);
  }

  @Override
  public HashingHandler create(final String algorithmIdentifier, final byte[] salt) throws CipherException {
    return create(algorithmIdentifier, salt, null);
  }

  @Override
  public HashingHandler create(
      final String algorithmIdentifier,
      final byte[] salt,
      final Integer iterations) throws CipherException
  {
    return switch (algorithmIdentifier) {
      case KEY_FACTORY_ALGORITHM_SHA1 -> new HashingHandlerImpl(cryptoHelper,
          KEY_FACTORY_ALGORITHM_SHA1,
          salt,
          getIterationsForAlgorithm(iterations, DEFAULT_ITERATIONS_SHA1),
          KEY_LENGTH_SHA1);
      case KEY_FACTORY_ALGORITHM_SHA256 -> new HashingHandlerImpl(cryptoHelper,
          KEY_FACTORY_ALGORITHM_SHA256,
          salt,
          getIterationsForAlgorithm(iterations, DEFAULT_ITERATIONS_SHA256),
          KEY_LENGTH_SHA256);
      default -> throw new CipherException("Unsupported algorithm: " + algorithmIdentifier);
    };
  }

  /**
   * Determines the iterations to use based on priority:
   * 1. Explicit iterations parameter (from PHC format, JSON migration config, password/secrets service)
   * 2. Default iterations for the algorithm (SHA1: 1024, SHA256: 10000)
   */
  private static int getIterationsForAlgorithm(final Integer explicitIterations, final int defaultIterations) {
    return explicitIterations != null ? explicitIterations : defaultIterations;
  }
}
