
package com.globalart.astrum.ssl.plugin.internal.keystore;

import java.util.List;
import java.util.Optional;

import javax.annotation.Nullable;

import com.google.common.collect.ImmutableList;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.entity.EntityVersion;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * MyBatis {@link PersistentKeyStoreStorageManager} implementation.
 *
 * @since 3.21
 */
@Component
@Qualifier(KeyStoreManagerImpl.NAME)
public class KeyStoreStorageManagerImpl
    extends ConfigStoreSupport<KeyStoreDAO>
    implements PersistentKeyStoreStorageManager
{
  @Autowired
  public KeyStoreStorageManagerImpl(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Transactional
  @Nullable
  @Override
  public boolean exists(final String keyStoreName) {
    return dao().load(keyStoreName).isPresent();
  }

  @Transactional
  @Override
  public Optional<KeyStoreData> load(final String keyStoreName) {
    postCommitEvent(() -> event(keyStoreName));
    return dao().load(keyStoreName);
  }

  @Transactional
  @Override
  public void save(final KeyStoreData data) {
    postCommitEvent(() -> event(data.getName()));
    dao().save(data);
  }

  @Transactional
  @Override
  public List<KeyStoreData> browse() {
    return ImmutableList.copyOf(dao().browse());
  }

  private static KeyStoreDataEvent event(final String keyStoreName) {
    // trigger invalidation of TrustStoreImpl context
    return new KeyStoreDataEvent()
    {
      @Override
      public boolean isLocal() {
        return true;
      }

      @Override
      public EntityVersion getVersion() {
        return null;
      }

      @Override
      public String getRemoteNodeId() {
        return null;
      }

      @Override
      public String getKeyStoreName() {
        return keyStoreName;
      }
    };
  }
}
