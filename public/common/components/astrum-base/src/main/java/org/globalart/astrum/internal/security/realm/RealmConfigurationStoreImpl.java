
package org.globalart.astrum.internal.security.realm;

import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.security.realm.RealmConfiguration;
import org.globalart.astrum.security.realm.RealmConfigurationStore;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * MyBatis {@link RealmConfigurationStore} implementation.
 *
 * @since 3.21
 */
@Component
@Qualifier("mybatis")
public class RealmConfigurationStoreImpl
    extends ConfigStoreSupport<RealmConfigurationDAO>
    implements RealmConfigurationStore
{
  @Autowired
  public RealmConfigurationStoreImpl(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Override
  public RealmConfiguration newEntity() {
    return new RealmConfigurationData();
  }

  @Transactional
  @Override
  public RealmConfiguration load() {
    return dao().get().orElse(null);
  }

  @Transactional
  @Override
  public void save(final RealmConfiguration configuration) {
    postCommitEvent(() -> new RealmConfigurationChangedEvent((RealmConfigurationData) configuration));
    dao().set((RealmConfigurationData) configuration);
  }
}
