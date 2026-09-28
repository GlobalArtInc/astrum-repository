
package com.globalart.astrum.ssl.plugin.internal.keystore;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.security.KeyStore;
import java.security.KeyStoreException;
import java.security.NoSuchAlgorithmException;
import java.security.cert.CertificateException;

import org.globalart.astrum.ssl.spi.KeyStoreStorage;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * MyBatis {@link KeyStoreStorage} implementation.
 *
 * @since 3.21
 */
public class KeyStoreStorageImpl
    implements KeyStoreStorage
{
  private final PersistentKeyStoreStorageManager storage;

  private final String keyStoreName;

  public KeyStoreStorageImpl(final PersistentKeyStoreStorageManager storage, final String keyStoreName) {
    this.storage = checkNotNull(storage);
    this.keyStoreName = checkNotNull(keyStoreName);
  }

  @Override
  public boolean exists() {
    return storage.exists(keyStoreName);
  }

  @Override
  public boolean modified() {
    return false; // we don't track the external version at the moment
  }

  @Override
  public void load(
      final KeyStore keyStore,
      final char[] password) throws NoSuchAlgorithmException, CertificateException, IOException
  {
    try (ByteArrayInputStream in = load()) {
      keyStore.load(in, password);
    }
  }

  @Override
  public void save(
      final KeyStore keyStore,
      final char[] password) throws KeyStoreException, NoSuchAlgorithmException, CertificateException, IOException
  {
    try (ByteArrayOutputStream out = new ByteArrayOutputStream(16 * 1024)) {
      keyStore.store(out, password);
      save(out);
    }
  }

  private void save(final ByteArrayOutputStream out) {
    KeyStoreData data = new KeyStoreData();
    data.setName(keyStoreName);
    data.setBytes(out.toByteArray());
    storage.save(data);
  }

  private ByteArrayInputStream load() {
    return storage.load(keyStoreName)
        .map(KeyStoreData::getBytes)
        .map(ByteArrayInputStream::new)
        .orElseThrow(() -> new IllegalStateException("key store %s does not exist".formatted(keyStoreName)));
  }
}
