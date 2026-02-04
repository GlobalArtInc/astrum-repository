
package org.globalart.astrum.repository.content.blobstore.metrics.upgrade;

import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsEntity;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.datastore.api.DuplicateKeyException;
import org.globalart.astrum.repository.content.blobstore.metrics.BlobStoreMetricsDAO;
import org.globalart.astrum.upgrade.datastore.UpgradeConfigStoreSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.UPGRADE;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

/**
 * UPGRADE-phase-safe equivalent of {@code BlobStoreMetricsStore} for the {@code blob_store_metrics} table.
 *
 * <p>
 * {@code BlobStoreMetricsStoreImpl} extends {@code ConfigStoreSupport} (which injects the EVENTS-phase
 * {@code EventManager}) and so cannot be injected into UPGRADE-phase migration steps. This store reaches
 * the same {@link BlobStoreMetricsDAO} through a {@link DataSession} (reusing its SQL and type handlers)
 * without that dependency.
 * </p>
 */
@Component
@ManagedLifecycle(phase = UPGRADE)
public final class UpgradeBlobStoreMetricsStore
    extends UpgradeConfigStoreSupport
{
  @Autowired
  public UpgradeBlobStoreMetricsStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  /**
   * Returns the stored metrics for the named blob store, or {@code null} if none exist yet.
   */
  public BlobStoreMetricsEntity get(final String blobStoreName) {
    try (DataSession<?> session = sessionSupplier.openSession(DEFAULT_DATASTORE_NAME)) {
      return session.access(BlobStoreMetricsDAO.class).get(blobStoreName);
    }
  }

  /**
   * Creates the initial (zeroed) metrics row for the named blob store if it does not already exist. Opens
   * its own session and commits before returning. Mirrors {@code BlobStoreMetricsStoreImpl}: pre-checks
   * existence and absorbs a concurrent-insert {@link DuplicateKeyException} (an HA race between nodes is not
   * an error) so a re-run cannot throw on the raw {@code INSERT}.
   */
  public void initializeMetrics(final String blobStoreName) {
    try (DataSession<?> session = sessionSupplier.openSession(DEFAULT_DATASTORE_NAME)) {
      initializeMetricsIfAbsent(session.access(BlobStoreMetricsDAO.class), blobStoreName);
      session.getTransaction().commit();
    }
  }

  /**
   * Persists the given metrics values ({@code BlobStoreMetricsDAO.updateMetrics} is additive). Opens its own
   * session and commits before returning.
   */
  public void updateMetrics(final BlobStoreMetricsEntity blobStoreMetricsEntity) {
    try (DataSession<?> session = sessionSupplier.openSession(DEFAULT_DATASTORE_NAME)) {
      session.access(BlobStoreMetricsDAO.class).updateMetrics(blobStoreMetricsEntity);
      session.getTransaction().commit();
    }
  }

  /**
   * Initializes the (zeroed) row if absent and applies the given metrics in a <em>single</em> transaction,
   * committing once. If the update fails, the initialize is rolled back too — so a partial failure leaves no
   * zeroed row behind for the caller's {@code get(name) == null} filter to then skip forever. Use this for
   * the initialize-then-update flow rather than calling {@link #initializeMetrics}/{@link #updateMetrics}
   * separately.
   *
   * <p>
   * Unlike {@link #initializeMetrics} (its own transaction), this does <em>not</em> swallow a
   * {@link DuplicateKeyException} around the INSERT: this step runs at the UPGRADE phase under Flyway's
   * cluster-wide schema-history lock (single-writer), so no peer node inserts the row concurrently. On
   * PostgreSQL a failed INSERT would abort the whole transaction ({@code 25P02}), so swallowing it could not
   * recover the following {@code updateMetrics} anyway; and because {@code updateMetrics} is additive,
   * swallowing a concurrent insert would double-count against the peer's row. The {@code get() == null}
   * pre-check inside this single transaction is sufficient given the single-writer guarantee.
   * </p>
   */
  public void initializeAndUpdateMetrics(final BlobStoreMetricsEntity blobStoreMetricsEntity) {
    try (DataSession<?> session = sessionSupplier.openSession(DEFAULT_DATASTORE_NAME)) {
      BlobStoreMetricsDAO dao = session.access(BlobStoreMetricsDAO.class);
      if (dao.get(blobStoreMetricsEntity.getBlobStoreName()) == null) {
        dao.initializeMetrics(blobStoreMetricsEntity.getBlobStoreName());
      }
      dao.updateMetrics(blobStoreMetricsEntity);
      session.getTransaction().commit();
    }
  }

  private void initializeMetricsIfAbsent(final BlobStoreMetricsDAO dao, final String blobStoreName) {
    if (dao.get(blobStoreName) == null) {
      try {
        dao.initializeMetrics(blobStoreName);
      }
      catch (DuplicateKeyException e) {
        // HA race: another node initialized the row between our get() and insert. Not an error.
        log.debug("Metrics row for '{}' already initialized concurrently; skipping insert", blobStoreName, e);
      }
    }
  }
}
