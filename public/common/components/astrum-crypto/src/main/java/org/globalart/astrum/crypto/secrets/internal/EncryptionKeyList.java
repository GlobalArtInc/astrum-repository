
package org.globalart.astrum.crypto.secrets.internal;

import java.util.List;

/**
 * Simple POJO representing the structure present in the encryption keys JSON file
 */
public class EncryptionKeyList
{
  private String active;

  private List<SecretEncryptionKey> keys;

  private FixedEncryption fixedEncryption;

  private FixedEncryption previousFixedEncryption;

  public String getActive() {
    return active;
  }

  public void setActive(final String active) {
    this.active = active;
  }

  public List<SecretEncryptionKey> getKeys() {
    return keys;
  }

  public void setKeys(final List<SecretEncryptionKey> keys) {
    this.keys = keys;
  }

  public FixedEncryption getFixedEncryption() {
    return fixedEncryption;
  }

  public void setFixedEncryption(final FixedEncryption fixedEncryption) {
    this.fixedEncryption = fixedEncryption;
  }

  public FixedEncryption getPreviousFixedEncryption() {
    return previousFixedEncryption;
  }

  public void setPreviousFixedEncryption(final FixedEncryption previousFixedEncryption) {
    this.previousFixedEncryption = previousFixedEncryption;
  }

  public static class SecretEncryptionKey
  {
    private static final String DEFAULT_KEY = "key";

    private String id;

    private String key;

    public SecretEncryptionKey() {
    }

    public SecretEncryptionKey(final String id, final String key) {
      this.id = id;
      this.key = key;
    }

    public SecretEncryptionKey(final String key) {
      this.key = key;
      this.id = DEFAULT_KEY;
    }

    public String getId() {
      return id;
    }

    public void setId(final String id) {
      this.id = id;
    }

    public String getKey() {
      return key;
    }

    public void setKey(final String key) {
      this.key = key;
    }
  }

  public static class FixedEncryption
  {
    private String keyId;

    private String salt;

    private String iv;

    private Integer keyIterations;

    public FixedEncryption() {
    }

    public FixedEncryption(final String keyId, final String salt, final String iv) {
      this.keyId = keyId;
      this.salt = salt;
      this.iv = iv;
    }

    public FixedEncryption(final String keyId, final String salt, final String iv, final Integer keyIterations) {
      this.keyId = keyId;
      this.salt = salt;
      this.iv = iv;
      this.keyIterations = keyIterations;
    }

    public String getKeyId() {
      return keyId;
    }

    public void setKeyId(final String keyId) {
      this.keyId = keyId;
    }

    public String getSalt() {
      return salt;
    }

    public void setSalt(final String salt) {
      this.salt = salt;
    }

    public String getIv() {
      return iv;
    }

    public void setIv(final String iv) {
      this.iv = iv;
    }

    public Integer getKeyIterations() {
      return keyIterations;
    }

    public void setKeyIterations(final Integer keyIterations) {
      this.keyIterations = keyIterations;
    }
  }
}
