
package org.globalart.astrum.crypto.internal;

import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.security.SecureRandom;
import java.security.spec.InvalidKeySpecException;
import javax.crypto.SecretKey;
import javax.crypto.SecretKeyFactory;
import javax.crypto.spec.PBEKeySpec;

import org.globalart.astrum.crypto.CryptoHelper;
import org.globalart.astrum.crypto.internal.error.CipherException;
import org.globalart.astrum.crypto.secrets.EncryptedSecret;
import org.globalart.astrum.crypto.HashingHandler;

import com.google.common.collect.ImmutableMap;
import org.springframework.lang.Nullable;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.crypto.internal.EncryptionHelper.KEY_ITERATION_PHC;
import static org.globalart.astrum.crypto.internal.EncryptionHelper.KEY_LEN_PHC;
import static org.globalart.astrum.crypto.internal.EncryptionHelper.fromBase64;
import static org.globalart.astrum.crypto.internal.EncryptionHelper.toBase64;

public class HashingHandlerImpl
    implements HashingHandler
{
  private static final int SALT_SIZE = 16;

  private final SecretKeyFactory keyFactory;

  private final CryptoHelper cryptoHelper;

  private final String algorithm;

  private final byte[] salt;

  private final int keyIterations;

  private final int keyLength;

  public HashingHandlerImpl(
      final CryptoHelper cryptoHelper,
      final String astrumPasswordAlgorithm,
      @Nullable final byte[] salt,
      final int keyIterations,
      final int keyLength) throws CipherException
  {
    this.cryptoHelper = cryptoHelper;
    try {
      this.keyFactory = cryptoHelper.createSecretKeyFactory(astrumPasswordAlgorithm);
    }
    catch (NoSuchAlgorithmException e) {
      throw new CipherException(e.getMessage(), e);
    }
    this.algorithm = astrumPasswordAlgorithm;
    this.salt = salt == null ? generateRandomBytes(SALT_SIZE) : salt;
    this.keyIterations = keyIterations;
    this.keyLength = keyLength;
  }

  public EncryptedSecret hash(final char[] valueToHash) throws CipherException {
    try {
      byte[] encodedHash = generateHash(valueToHash, salt);

      return new EncryptedSecret(algorithm, null, toBase64(salt), toBase64(encodedHash),
          ImmutableMap.of(KEY_ITERATION_PHC, String.valueOf(this.keyIterations),
              KEY_LEN_PHC, String.valueOf(this.keyLength)));
    }
    catch (InvalidKeySpecException e) {
      throw new CipherException(e.getMessage(), e);
    }
  }

  public boolean verify(
      final char[] input,
      final String expectedValue) throws CipherException, InvalidKeySpecException
  {
    checkNotNull(input);
    checkNotNull(expectedValue);

    EncryptedSecret parsedExpectedHash = EncryptedSecret.parse(expectedValue);
    byte[] inputHash = this.generateHash(input, fromBase64(parsedExpectedHash.getSalt()));
    byte[] expectedHash = fromBase64(parsedExpectedHash.getValue());

    return MessageDigest.isEqual(inputHash, expectedHash);
  }

  private byte[] generateHash(final char[] input, final byte[] salt) throws InvalidKeySpecException {
    PBEKeySpec spec = new PBEKeySpec(input, salt, this.keyIterations, this.keyLength);
    try {
      SecretKey secretKey = keyFactory.generateSecret(spec);
      return secretKey.getEncoded();
    }
    finally {
      spec.clearPassword();
    }
  }

  private byte[] generateRandomBytes(final int size) {
    SecureRandom localRandom = cryptoHelper.createSecureRandom();
    byte[] bytes = new byte[size];
    localRandom.nextBytes(bytes);
    return bytes;
  }
}
