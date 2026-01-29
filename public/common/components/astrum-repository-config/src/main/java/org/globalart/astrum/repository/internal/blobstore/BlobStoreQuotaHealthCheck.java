
package org.globalart.astrum.repository.internal.blobstore;

import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;
import java.util.stream.StreamSupport;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaResult;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaService;

import com.codahale.metrics.health.HealthCheck;
import com.google.common.base.Preconditions;
import com.google.common.collect.Iterators;

import static java.lang.String.format;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Inform on the health of all BlobStores based on their configured soft quota.
 */
@Component
@Qualifier("Blob Stores Quota")
public class BlobStoreQuotaHealthCheck
    extends HealthCheck
{
  private final Provider<BlobStoreManager> blobStoreManagerProvider;

  private final Provider<BlobStoreQuotaService> quotaServiceProvider;

  @Autowired
  public BlobStoreQuotaHealthCheck(
      final Provider<BlobStoreManager> blobStoreManagerProvider,
      final Provider<BlobStoreQuotaService> quotaServiceProvider)
  {
    this.blobStoreManagerProvider = Preconditions.checkNotNull(blobStoreManagerProvider);
    this.quotaServiceProvider = Preconditions.checkNotNull(quotaServiceProvider);
  }

  @Override
  protected Result check() {
    Iterable<BlobStore> blobstoreItr = blobStoreManagerProvider.get().browse();
    final List<String> violationMessages = StreamSupport.stream(blobstoreItr.spliterator(), false)
        .map(blobStore -> quotaServiceProvider.get().checkQuota(blobStore))
        .filter(Objects::nonNull)
        .filter(BlobStoreQuotaResult::isViolation)
        .map(BlobStoreQuotaResult::getMessage)
        .collect(Collectors.toList());

    String message = format("%s/%s blob stores violating their quota<br>%s", violationMessages.size(),
        Iterators.size(blobStoreManagerProvider.get().browse().iterator()),
        String.join("<br>", violationMessages));

    return violationMessages.isEmpty() ? Result.healthy(message) : Result.unhealthy(message);
  }
}
