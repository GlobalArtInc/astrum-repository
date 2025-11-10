
package org.globalart.astrum.crypto.secrets;

import java.util.Base64;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.crypto.internal.PbeCipherFactory;
import org.globalart.astrum.crypto.internal.PbeCipherFactory.PbeCipher;
import org.globalart.astrum.crypto.secrets.internal.EncryptionKeyList.SecretEncryptionKey;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.nio.charset.StandardCharsets.UTF_8;
import org.springframework.stereotype.Component;

@Component
public class EncryptDecryptService
    implements EncryptDecrypt<String, String, String>
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final PbeCipherFactory pbeCipherFactory;

  private String secret = "changeme";

  @Autowired
  public EncryptDecryptService(final PbeCipherFactory pbeCipherFactory) {
    this.pbeCipherFactory = checkNotNull(pbeCipherFactory);
  }

  @Override
  public String encode(String encryptedString) {
    return new String(Base64.getEncoder().encode(encryptedString.getBytes(UTF_8)), UTF_8);
  }

  @Override
  public String decode(String encodedString) {
    return new String(Base64.getDecoder().decode(encodedString.getBytes(UTF_8)), UTF_8);
  }

  @Override
  public String encrypt(String stringToEncrypt) {
    try {
      SecretEncryptionKey secretKey = new SecretEncryptionKey(secret);
      PbeCipher pbeCipher = pbeCipherFactory.create(secretKey);
      return pbeCipher.encrypt(stringToEncrypt.getBytes(UTF_8)).toPhcString();
    }
    catch (Exception e) {
      log.error("Failed to encrypt", e);
      throw new RuntimeException("Unable to encrypt.", e);
    }
  }

  @Override
  public String decrypt(String stringToDecrypt) {
    try {
      SecretEncryptionKey secretKey = new SecretEncryptionKey(secret);
      PbeCipher pbeCipher = pbeCipherFactory.create(secretKey, stringToDecrypt);
      return new String(pbeCipher.decrypt(), UTF_8);
    }
    catch (Exception e) {
      log.error("Failed to decrypt", e);
      throw new RuntimeException("Unable to decrypt.", e);
    }
  }

  public void setSecret(String secret) {
    this.secret = secret;
  }
}
