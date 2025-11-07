
package org.globalart.astrum.crypto.internal;

import java.security.MessageDigest;
import java.security.SecureRandom;
import java.security.spec.AlgorithmParameterSpec;
import javax.crypto.Cipher;
import javax.crypto.Mac;
import javax.crypto.SecretKey;
import javax.crypto.spec.IvParameterSpec;
import javax.crypto.spec.SecretKeySpec;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.crypto.CryptoHelper;
import org.globalart.astrum.crypto.internal.error.CipherException;
import org.globalart.astrum.crypto.secrets.EncryptedSecret;
import org.globalart.astrum.crypto.HashingHandler;
import org.globalart.astrum.crypto.secrets.internal.EncryptionKeyList.SecretEncryptionKey;

import com.google.common.base.Throwables;
import com.google.common.collect.ImmutableMap;
import org.bouncycastle.util.encoders.Hex;
import org.springframework.beans.factory.annotation.Value;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.ASTRUM_SECURITY_SECRETS_ALGORITHM_NAMED_VALUE;
import static org.globalart.astrum.common.app.FeatureFlags.ASTRUM_SECURITY_SECRETS_ITERATIONS_NAMED_VALUE;
import static org.globalart.astrum.crypto.internal.EncryptionHelper.KEY_ITERATION_PHC;
import static org.globalart.astrum.crypto.internal.EncryptionHelper.KEY_LEN_PHC;
import static org.globalart.astrum.crypto.internal.EncryptionHelper.fromBase64;
import static org.globalart.astrum.crypto.internal.EncryptionHelper.toBase64;
import static org.globalart.astrum.crypto.internal.HashingHandlerFactoryImpl.KEY_FACTORY_ALGORITHM_SHA1;

import org.springframework.stereotype.Component;

/**
 * Default implementation for {@link PbeCipherFactory} . provides a simple cipher supporting PHC string format
 */
@Component
public class PbeCipherFactoryImpl
    implements PbeCipherFactory
{
  private final CryptoHelper cryptoHelper;

  private final HashingHandlerFactory hashingHandlerFactory;

  private final String astrumSecretsAlgorithm;

  private final Integer configuredSecretsIterations;

  @Autowired
  public PbeCipherFactoryImpl(
      final CryptoHelper cryptoHelper,
      final HashingHandlerFactory hashingHandlerFactory,
      final @Value(ASTRUM_SECURITY_SECRETS_ALGORITHM_NAMED_VALUE) String astrumSecretsAlgorithm,
      final @Value(ASTRUM_SECURITY_SECRETS_ITERATIONS_NAMED_VALUE) Integer configuredSecretsIterations)
  {
    this.cryptoHelper = checkNotNull(cryptoHelper);
    this.hashingHandlerFactory = checkNotNull(hashingHandlerFactory);
    this.astrumSecretsAlgorithm = astrumSecretsAlgorithm;
    this.configuredSecretsIterations = configuredSecretsIterations;
  }

  @Override
  public PbeCipher create(final SecretEncryptionKey secretEncryptionKey) throws CipherException {
    return doCreate(secretEncryptionKey, null, null, null, null);
  }

  @Override
  public PbeCipher create(
      final SecretEncryptionKey secretEncryptionKey,
      final String encryptedSecret) throws CipherException
  {
    return doCreate(secretEncryptionKey, encryptedSecret, null, null, null);
  }

  @Override
  public PbeCipher create(
      final SecretEncryptionKey secretEncryptionKey,
      final String salt,
      final String iv,
      final Integer iterations) throws CipherException
  {
    return doCreate(secretEncryptionKey, null, salt, iv, iterations);
  }

  private PbeCipher doCreate(
      final SecretEncryptionKey secretEncryptionKey,
      final String encryptedSecret,
      final String salt,
      final String iv,
      final Integer iterations)
  {
    checkNotNull(secretEncryptionKey);
    EncryptedSecret storedEncryptedSecret = null;
    String algorithm = astrumSecretsAlgorithm;
    boolean isDefaultCipher = true;
    Integer iterationsToBeUsed = iterations;

    byte[] saltToBeUsed = salt != null ? salt.getBytes() : null;

    if (encryptedSecret != null) {
      storedEncryptedSecret = EncryptedSecret.parse(encryptedSecret);
      algorithm = getAlgorithm(storedEncryptedSecret);
      isDefaultCipher = astrumSecretsAlgorithm.equals(algorithm);
      saltToBeUsed = fromBase64(storedEncryptedSecret.getSalt());

      if (iterationsToBeUsed == null) {
        String iterationsStr = storedEncryptedSecret.getAttributes().get(KEY_ITERATION_PHC);
        if (iterationsStr != null) {
          try {
            iterationsToBeUsed = Integer.parseInt(iterationsStr);
            isDefaultCipher = isDefaultCipher && (configuredSecretsIterations == null
                || configuredSecretsIterations.equals(iterationsToBeUsed));
          }
          catch (NumberFormatException e) {
          }
        }

      }
    }

    // Use configured iterations from astrum.properties if no explicit iterations provided
    if (iterationsToBeUsed == null && configuredSecretsIterations != null) {
      iterationsToBeUsed = configuredSecretsIterations;
    }

    HashingHandler hashingHandler = hashingHandlerFactory.create(algorithm, saltToBeUsed, iterationsToBeUsed);

    return new PbeCipherImpl(cryptoHelper, hashingHandler, secretEncryptionKey, storedEncryptedSecret, isDefaultCipher,
        iv);
  }

  private static String getAlgorithm(final EncryptedSecret storedEncryptedSecret) {
    String algorithm = storedEncryptedSecret.getAlgorithm();
    // this is for backwards compatibility since it was being used as PHC identifier
    if (PbeCipherImpl.ALGORITHM.equals(algorithm)) {
      algorithm = KEY_FACTORY_ALGORITHM_SHA1;
    }
    return algorithm;
  }

  /**
   * Abstract {@link PbeCipher} implementation, defines all the logic with no configuration.
   */
  static class PbeCipherImpl
      implements PbeCipher
  {
    private static final Logger log = LoggerFactory.getLogger(PbeCipherImpl.class);

    private static final String ALGORITHM = "AES/CBC/PKCS5Padding";

    private static final String KEY_ALGORITHM = "AES";

    private static final String HMAC_ALGORITHM = "HmacSHA256";

    private static final int IV_SIZE = 16;

    private static final String IV_PHC = "iv";

    private static final String HMAC_PHC = "hmac";

    private final CryptoHelper cryptoHelper;

    private final SecretEncryptionKey secretEncryptionKey;

    private final HashingHandler hashingHandler;

    private final EncryptedSecret storedEncryptedSecret;

    private final boolean isDefaultCipher;

    private final byte[] iv;

    PbeCipherImpl(
        final CryptoHelper cryptoHelper,
        final HashingHandler hashingHandler,
        final SecretEncryptionKey secretEncryptionKey,
        final EncryptedSecret storedEncryptedSecret,
        final boolean isDefaultCipher,
        final String iv) throws CipherException
    {
      this.cryptoHelper = cryptoHelper;
      this.hashingHandler = hashingHandler;

      this.secretEncryptionKey = secretEncryptionKey;
      this.storedEncryptedSecret = storedEncryptedSecret;
      this.isDefaultCipher = isDefaultCipher;
      if (storedEncryptedSecret != null) {
        String ivStored = storedEncryptedSecret.getAttributes().get(IV_PHC);
        this.iv = ivStored != null ? Hex.decode(ivStored) : null;
      }
      else {
        this.iv = iv == null ? generateRandomBytes(IV_SIZE) : iv.getBytes();
      }
    }

    @Override
    public boolean isDefaultCipher() {
      return this.isDefaultCipher;
    }

    @Override
    public EncryptedSecret encrypt(final byte[] bytes) throws CipherException {
      EncryptedSecret encryptedSecretHash = hashingHandler.hash(secretEncryptionKey.getKey().toCharArray());

      String saltBase64 = encryptedSecretHash.getSalt();
      byte[] derivedKey = fromBase64(encryptedSecretHash.getValue());
      SecretKey secretKey = new SecretKeySpec(derivedKey, KEY_ALGORITHM);
      AlgorithmParameterSpec paramSpec = new IvParameterSpec(this.iv); // NOSONAR
      byte[] encrypted = transform(Cipher.ENCRYPT_MODE, secretKey, paramSpec, bytes);
      // ROLLBACK RISK: Secrets encrypted with this method include an 'hmac' PHC attribute that
      // did not exist in previous releases. Rolling back to an older version will cause
      // decrypt() failures for any secret re-encrypted after this upgrade, because old code
      // does not know how to handle the 'hmac' field and will likely reject the ciphertext.
      // Before rolling back, ensure all encrypted secrets have been re-decrypted with old code
      // or accept that re-encryption from plaintext sources will be required.
      byte[] hmac = computeHmac(derivedKey, this.iv, encrypted);

      return new EncryptedSecret(encryptedSecretHash.getAlgorithm(), null, saltBase64, toBase64(encrypted),
          ImmutableMap.of(IV_PHC, Hex.toHexString(this.iv),
              KEY_ITERATION_PHC, encryptedSecretHash.getAttributes().get(KEY_ITERATION_PHC),
              KEY_LEN_PHC, encryptedSecretHash.getAttributes().get(KEY_LEN_PHC),
              HMAC_PHC, Hex.toHexString(hmac)));
    }

    @Override
    public byte[] decrypt() throws CipherException {
      byte[] encrypted = fromBase64(storedEncryptedSecret.getValue());
      EncryptedSecret encryptedSecretHash = hashingHandler.hash(secretEncryptionKey.getKey().toCharArray());
      // NOTE: derivedKey is used for both AES-CBC encryption and HMAC-SHA256 authentication.
      // Key reuse across algorithms is a known trade-off; the two operations are structurally
      // independent (encrypt-then-MAC), so there is no practical cross-domain attack for this
      // configuration. A future improvement would be to derive separate sub-keys via HKDF.
      byte[] derivedKey = fromBase64(encryptedSecretHash.getValue());

      String storedHmac = storedEncryptedSecret.getAttributes().get(HMAC_PHC);
      if (storedHmac != null) {
        byte[] expectedHmac = computeHmac(derivedKey, iv, encrypted);
        if (!MessageDigest.isEqual(expectedHmac, Hex.decode(storedHmac))) {
          throw new CipherException("HMAC verification failed - invalid password or corrupted data");
        }
      }

      SecretKey secretKey = new SecretKeySpec(derivedKey, KEY_ALGORITHM);
      AlgorithmParameterSpec paramSpec = new IvParameterSpec(iv); // NOSONAR
      return transform(Cipher.DECRYPT_MODE, secretKey, paramSpec, encrypted);
    }

    /**
     * Decrypts a raw ciphertext byte array using the configured key and IV.
     *
     * <p>
     * <strong>Note:</strong> This overload does NOT perform HMAC verification because the
     * caller supplies the raw ciphertext directly (without a stored PHC envelope). The caller
     * is responsible for verifying integrity before invoking this method.
     * </p>
     *
     * @deprecated Prefer {@link #decrypt()} which reads from the stored PHC envelope and performs
     *             HMAC verification automatically. Only use this overload when you have a raw ciphertext
     *             with no PHC envelope and have already verified integrity by other means.
     */
    @Deprecated
    @Override
    public byte[] decrypt(byte[] encrypted) throws CipherException {
      EncryptedSecret encryptedSecretHash = hashingHandler.hash(secretEncryptionKey.getKey().toCharArray());

      SecretKey secretKey = new SecretKeySpec(fromBase64(encryptedSecretHash.getValue()), KEY_ALGORITHM);
      AlgorithmParameterSpec paramSpec = new IvParameterSpec(iv); // NOSONAR

      return transform(Cipher.DECRYPT_MODE, secretKey, paramSpec, encrypted);
    }

    private byte[] computeHmac(final byte[] key, final byte[] iv, final byte[] ciphertext) throws CipherException {
      try {
        Mac mac;
        try {
          mac = Mac.getInstance(HMAC_ALGORITHM, cryptoHelper.getProvider());
        }
        catch (Exception e) {
          log.warn("BouncyCastle provider unavailable for HMAC computation, falling back to default JCE provider: {}",
              e.getMessage());
          mac = Mac.getInstance(HMAC_ALGORITHM);
        }
        mac.init(new SecretKeySpec(key, HMAC_ALGORITHM));
        mac.update(iv);
        return mac.doFinal(ciphertext);
      }
      catch (Exception e) {
        Throwables.throwIfUnchecked(e);
        throw new CipherException("Failed to compute HMAC", e);
      }
    }

    private byte[] generateRandomBytes(final int size) {
      SecureRandom localRandom = cryptoHelper.createSecureRandom();
      byte[] bytes = new byte[size];
      localRandom.nextBytes(bytes);
      return bytes;
    }

    private byte[] transform(
        final int mode,
        final SecretKey secretKey,
        final AlgorithmParameterSpec paramSpec,
        final byte[] bytes) throws CipherException
    {
      try {
        Cipher cipher = cryptoHelper.createCipher(ALGORITHM);
        cipher.init(mode, secretKey, paramSpec);
        return cipher.doFinal(bytes, 0, bytes.length);
      }
      catch (Exception e) {
        Throwables.throwIfUnchecked(e);
        throw new CipherException(e.getMessage(), e);
      }
    }
  }
}
