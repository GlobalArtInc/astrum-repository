
package org.globalart.astrum.datastore.mybatis;

import java.util.Optional;

import org.globalart.astrum.crypto.LegacyCipherFactory.PbeCipher;
import org.globalart.astrum.crypto.internal.CryptoHelperImpl;
import org.globalart.astrum.crypto.internal.HashingHandlerFactoryImpl;
import org.globalart.astrum.crypto.internal.PbeCipherFactory;
import org.globalart.astrum.crypto.internal.PbeCipherFactoryImpl;
import org.globalart.astrum.crypto.secrets.EncryptedSecret;
import org.globalart.astrum.crypto.secrets.internal.EncryptionKeyList.FixedEncryption;
import org.globalart.astrum.crypto.secrets.internal.EncryptionKeyList.SecretEncryptionKey;
import org.globalart.astrum.crypto.secrets.internal.EncryptionKeySource;
import org.globalart.astrum.crypto.secrets.internal.EncryptionKeySourceImpl;
import org.globalart.astrum.security.PasswordHelper;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.crypto.internal.EncryptionHelper.fromBase64;

/**
 * Database cipher shared between all MyBatis handlers that want to encrypt data at rest.
 * <p>
 * Note this is different from nested password attributes which should be encrypted with
 * {@link PasswordHelper}, this is about encrypting the entire content of a database cell.
 *
 * @since 3.21
 */
@Component
@Qualifier("mybatis")
final class MyBatisCipher
    implements PbeCipher
{
  private final PbeCipherFactory.PbeCipher pbeCipher;

  @Autowired
  MyBatisCipher(
      @Value("${astrum.mybatis.cipher.password:changeme}") final String password,
      @Value("${astrum.mybatis.cipher.salt:changeme}") final String salt,
      @Value("${astrum.mybatis.cipher.iv:0123456789ABCDEF}") final String iv,
      final PbeCipherFactory pbeCipherFactory,
      final EncryptionKeySource encryptionKeySource)
  {
    String saltValue = salt;
    String ivValue = iv;
    SecretEncryptionKey keyForEncryption = new SecretEncryptionKey(null, password);
    Integer iterationsForEncryption = null;

    Optional<FixedEncryption> fixedEncryptionConfiguration = encryptionKeySource.getFixedEncryption();
    if (fixedEncryptionConfiguration.isPresent()) {
      FixedEncryption encryptionConfig = fixedEncryptionConfiguration.get();
      keyForEncryption = encryptionConfig.getKeyId() != null
          ? encryptionKeySource.getKey(encryptionConfig.getKeyId()).orElseThrow()
          : keyForEncryption;
      saltValue = encryptionConfig.getSalt() != null ? encryptionConfig.getSalt() : saltValue;
      ivValue = encryptionConfig.getIv() != null ? encryptionConfig.getIv() : ivValue;
    }
    this.pbeCipher =
        checkNotNull(pbeCipherFactory).create(keyForEncryption, saltValue, ivValue, iterationsForEncryption);
  }

  /**
   * Static configuration for testing purposes.
   */
  @VisibleForTesting
  MyBatisCipher() {
    this("changeme", "changeme", "0123456789ABCDEF",
        new PbeCipherFactoryImpl(new CryptoHelperImpl(false),
            new HashingHandlerFactoryImpl(new CryptoHelperImpl(false)), "PBKDF2WithHmacSHA1", null),
        new EncryptionKeySourceImpl(null, null));
  }

  @Override
  public byte[] encrypt(final byte[] bytes) {
    EncryptedSecret encryptedSecret = pbeCipher.encrypt(bytes);
    String base64Value = encryptedSecret.getValue();
    return fromBase64(base64Value);
  }

  @Override
  public byte[] decrypt(final byte[] bytes) {
    return pbeCipher.decrypt(bytes);
  }
}
