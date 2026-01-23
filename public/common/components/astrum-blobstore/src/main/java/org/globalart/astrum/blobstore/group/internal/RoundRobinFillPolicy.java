
package org.globalart.astrum.blobstore.group.internal;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.group.BlobStoreGroup;
import org.globalart.astrum.blobstore.group.FillPolicy;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaResult;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaService;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.beans.factory.annotation.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static java.util.Collections.rotate;
import static org.globalart.astrum.common.app.FeatureFlags.BLOBSTORE_SKIP_ON_SOFTQUOTA_VIOLATION;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * {@link FillPolicy} that divides writes to member blob stores evenly based upon a round robin selection.
 *
 * @since 3.14
 */
@Component
@Qualifier(RoundRobinFillPolicy.TYPE)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RoundRobinFillPolicy
    implements FillPolicy
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String TYPE = "roundRobin";

  protected static final String NAME = "Round Robin";

  @VisibleForTesting
  AtomicInteger sequence = new AtomicInteger();

  @Autowired
  private BlobStoreQuotaService quotaService;

  @Autowired
  @Value("${" + BLOBSTORE_SKIP_ON_SOFTQUOTA_VIOLATION + ":false}")
  boolean skipOnSoftQuotaViolation;

  @Override
  public String getName() {
    return NAME;
  }

  @Override
  @Nullable
  public BlobStore chooseBlobStore(
      final BlobStoreGroup blobStoreGroup,
      final Map<String, String> headers)
  {
    return nextMember(blobStoreGroup.getMembers());
  }

  /**
   * Retrieves the next writable member in the group
   *
   * @param members of the BlobStoreGroup
   * @return the first writable {@link BlobStore} or null if none are writable
   */
  @Nullable
  private BlobStore nextMember(final List<BlobStore> members) {
    if (members.isEmpty()) {
      return null;
    }
    final int index = nextIndex() % members.size();
    log.trace("Using index {}", index);

    ArrayList<BlobStore> rotatedMembers = new ArrayList<>(members);
    rotate(rotatedMembers, index);
    return rotatedMembers.stream()
        .filter(BlobStore::isWritable)
        .filter(BlobStore::isStorageAvailable)
        .filter(skipOnSoftQuotaViolation ? this::hasNoQuotaViolation : s -> true)
        .findFirst()
        .orElse(null);
  }

  @VisibleForTesting
  int nextIndex() {
    return sequence.getAndUpdate(i -> {
      i += 1;
      return i < 0 ? 0 : i;
    });
  }

  private boolean hasNoQuotaViolation(final BlobStore blobStore) {
    BlobStoreQuotaResult result = quotaService.checkQuota(blobStore);
    if (result != null && result.isViolation()) {
      if (log.isTraceEnabled()) {
        log.info("Skipping blobStore {} due to soft-quota violation: {}", result.getBlobStoreName(),
            result.getMessage());
      }
      else {
        log.info("Skipping blobStore {} due to soft-quota violation", result.getBlobStoreName());
      }
      return false;
    }
    return true;
  }
}
