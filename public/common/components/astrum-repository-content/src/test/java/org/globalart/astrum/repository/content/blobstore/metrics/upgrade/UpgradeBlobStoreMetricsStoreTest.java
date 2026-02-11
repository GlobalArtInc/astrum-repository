
package org.globalart.astrum.repository.content.blobstore.metrics.upgrade;

import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsEntity;
import org.globalart.astrum.repository.content.blobstore.metrics.BlobStoreMetricsDAO;
import org.globalart.astrum.testdb.DataSessionConfiguration;
import org.globalart.astrum.testdb.DatabaseTest;
import org.globalart.astrum.testdb.TestDataSessionSupplier;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * Real-database tests for {@link UpgradeBlobStoreMetricsStore} against {@code blob_store_metrics}. These run
 * against H2 only; PostgreSQL parity is not exercised here and will be covered by the forthcoming
 * {@code UpgradeMatrixIT} in {@code astrum-integration-tests}. ({@code blob_store_metrics} has no JSON/JSONB
 * column, so there is no dialect branch in this store; the only PG-specific behaviour is the
 * single-writer/transaction handling of {@code initializeAndUpdateMetrics}, exercised there.)
 */
class UpgradeBlobStoreMetricsStoreTest
{
  @DataSessionConfiguration(daos = {BlobStoreMetricsDAO.class})
  TestDataSessionSupplier dataSessionSupplier;

  private UpgradeBlobStoreMetricsStore store() {
    return new UpgradeBlobStoreMetricsStore(dataSessionSupplier);
  }

  @DatabaseTest
  void get_missing_returnsNull() {
    assertThat(store().get("absent")).isNull();
  }

  @DatabaseTest
  void initialize_then_get_returnsZeroedEntity() {
    UpgradeBlobStoreMetricsStore store = store();
    store.initializeMetrics("bs1");

    BlobStoreMetricsEntity entity = store.get("bs1");
    assertThat(entity).isNotNull();
    assertThat(entity.getBlobStoreName()).isEqualTo("bs1");
    assertThat(entity.getBlobCount()).isZero();
    assertThat(entity.getTotalSize()).isZero();
  }

  @DatabaseTest
  void updateMetrics_isAdditiveAcrossCalls() {
    UpgradeBlobStoreMetricsStore store = store();
    store.initializeMetrics("bs2");

    store.updateMetrics(new BlobStoreMetricsEntity().setBlobStoreName("bs2").setTotalSize(100L).setBlobCount(5L));
    store.updateMetrics(new BlobStoreMetricsEntity().setBlobStoreName("bs2").setTotalSize(50L).setBlobCount(2L));

    BlobStoreMetricsEntity entity = store.get("bs2");
    // updateMetrics is additive (SET total_size = total_size + ?), so the two calls accumulate; this would
    // fail if the SQL were changed to replace semantics.
    assertThat(entity.getTotalSize()).isEqualTo(150L);
    assertThat(entity.getBlobCount()).isEqualTo(7L);
  }

  @DatabaseTest
  void initializeMetrics_isIdempotent() {
    UpgradeBlobStoreMetricsStore store = store();
    store.initializeMetrics("bs3");
    // second call must not throw (pre-existence check) nor reset the row
    store.initializeMetrics("bs3");
    store.updateMetrics(new BlobStoreMetricsEntity().setBlobStoreName("bs3").setTotalSize(10L).setBlobCount(1L));

    BlobStoreMetricsEntity entity = store.get("bs3");
    assertThat(entity.getTotalSize()).isEqualTo(10L);
    assertThat(entity.getBlobCount()).isEqualTo(1L);
  }

  @DatabaseTest
  void initializeAndUpdateMetrics_createsRowAndAppliesMetricsInOneCall() {
    UpgradeBlobStoreMetricsStore store = store();

    store.initializeAndUpdateMetrics(
        new BlobStoreMetricsEntity().setBlobStoreName("bs4").setTotalSize(200L).setBlobCount(8L));

    BlobStoreMetricsEntity entity = store.get("bs4");
    assertThat(entity).isNotNull();
    assertThat(entity.getTotalSize()).isEqualTo(200L);
    assertThat(entity.getBlobCount()).isEqualTo(8L);
  }
}
