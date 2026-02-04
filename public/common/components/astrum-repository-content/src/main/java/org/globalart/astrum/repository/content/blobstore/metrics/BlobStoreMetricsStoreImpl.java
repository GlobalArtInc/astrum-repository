
package org.globalart.astrum.repository.content.blobstore.metrics;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsEntity;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsStore;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.datastore.api.DuplicateKeyException;
import org.globalart.astrum.transaction.Transactional;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class BlobStoreMetricsStoreImpl
    extends ConfigStoreSupport<BlobStoreMetricsDAO>
    implements BlobStoreMetricsStore
{
  @Autowired
  protected BlobStoreMetricsStoreImpl(
      final DataSessionSupplier sessionSupplier)
  {
    super(sessionSupplier, BlobStoreMetricsDAO.class);
  }

  @Override
  @Transactional
  public void updateMetrics(final BlobStoreMetricsEntity blobStoreMetricsEntity) {
    dao().updateMetrics(blobStoreMetricsEntity);
  }

  @Override
  @Transactional
  public BlobStoreMetricsEntity get(final String blobStoreName) {
    return dao().get(blobStoreName);
  }

  @Override
  @Transactional
  public void remove(final String blobStoreName) {
    dao().remove(blobStoreName);
  }

  @Override
  @Transactional
  public void clearOperationMetrics(final String blobStoreName) {
    dao().clearOperationMetrics(blobStoreName);
  }

  @Override
  @Transactional
  public void clearCountMetrics(final String blobStoreName) {
    dao().clearCountMetrics(blobStoreName);
  }

  @Override
  @Transactional
  public void initializeMetrics(String blobStoreName) {
    try {
      if (dao().get(blobStoreName) == null) {
        dao().initializeMetrics(blobStoreName);
      }
    }
    catch (DuplicateKeyException e) {
      log.debug("Failed to initialize blobstore metrics as they are already initialized.",
          e); // this is likely an HA race condition between multiple nodes - this is not a problem
    }
  }

  @Override
  @Transactional
  public void rename(final String oldName, final String newName) {
    checkNotNull(oldName);
    checkNotNull(newName);
    dao().rename(oldName, newName);
  }
}
