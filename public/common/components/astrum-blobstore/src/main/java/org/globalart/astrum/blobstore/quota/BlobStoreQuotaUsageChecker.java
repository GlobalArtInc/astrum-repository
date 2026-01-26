
package org.globalart.astrum.blobstore.quota;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.common.scheduling.PeriodicJobService;
import org.globalart.astrum.common.scheduling.PeriodicJobService.PeriodicJob;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;

import org.springframework.beans.factory.annotation.Value;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;
import static org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport.createQuotaCheckJob;

/**
 * Checks blob store quota usage on a periodic basis.
 * <p>
 * Must be prototype-scoped because each instance maintains state specific to a single blob store
 * ({@link #blobStore} and {@link #quotaCheckingJob}). If singleton-scoped, concurrent blob store
 * initialization would fail since {@link #setBlobStore(BlobStore)} guards against re-initialization.
 *
 * @since 3.41
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class BlobStoreQuotaUsageChecker
    extends StateGuardLifecycleSupport
{
  protected final PeriodicJobService jobService;

  protected final int quotaCheckInterval;

  protected final BlobStoreQuotaService quotaService;

  protected BlobStore blobStore;

  protected PeriodicJob quotaCheckingJob;

  @Autowired
  public BlobStoreQuotaUsageChecker(
      final PeriodicJobService jobService,
      @Value("${astrum.blobstore.quota.warnIntervalSeconds:60}") final int quotaCheckInterval,
      final BlobStoreQuotaService quotaService)
  {
    this.jobService = checkNotNull(jobService);
    checkArgument(quotaCheckInterval > 0);
    this.quotaCheckInterval = quotaCheckInterval;
    this.quotaService = checkNotNull(quotaService);
  }

  @Override
  protected void doStart() throws Exception {
    jobService.startUsing();
    quotaCheckingJob = jobService.schedule(createQuotaCheckJob(blobStore, quotaService, log), quotaCheckInterval);
  }

  @Override
  protected void doStop() throws Exception {
    blobStore = null;
    quotaCheckingJob.cancel();
    quotaCheckingJob = null;
    jobService.stopUsing();
  }

  public void setBlobStore(final BlobStore blobStore) {
    checkState(this.blobStore == null, "Do not initialize twice");
    checkNotNull(blobStore);
    this.blobStore = blobStore;
  }
}
