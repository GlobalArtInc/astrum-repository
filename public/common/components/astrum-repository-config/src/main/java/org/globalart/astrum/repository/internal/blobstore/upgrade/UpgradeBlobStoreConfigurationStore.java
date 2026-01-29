
package org.globalart.astrum.repository.internal.blobstore.upgrade;

import java.util.List;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.internal.blobstore.BlobStoreConfigurationDAO;
import org.globalart.astrum.upgrade.datastore.UpgradeConfigStoreSupport;

import com.google.common.collect.ImmutableList;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.UPGRADE;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

/**
 * UPGRADE-phase-safe equivalent of {@code BlobStoreConfigurationStore} for the
 * {@code blob_store_configuration} table.
 *
 * <p>
 * {@code BlobStoreConfigurationStoreImpl} extends {@code ConfigStoreSupport} (which injects the
 * EVENTS-phase {@code EventManager}) and so cannot be injected into UPGRADE-phase migration steps. This
 * store reaches the same {@link BlobStoreConfigurationDAO} through a {@link DataSession} (reusing its SQL
 * and type handlers, so attributes are fully reconstructed) without that dependency.
 * </p>
 */
@Component
@ManagedLifecycle(phase = UPGRADE)
public final class UpgradeBlobStoreConfigurationStore
    extends UpgradeConfigStoreSupport
{
  @Autowired
  public UpgradeBlobStoreConfigurationStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  /**
   * Returns all stored blob store configurations.
   */
  public List<BlobStoreConfiguration> list() {
    try (DataSession<?> session = sessionSupplier.openSession(DEFAULT_DATASTORE_NAME)) {
      return ImmutableList.copyOf(session.access(BlobStoreConfigurationDAO.class).browse());
    }
  }
}
