
package org.globalart.astrum.blobstore.quota.internal;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.quota.BlobStoreQuota;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaResult;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.apache.commons.text.StringEscapeUtils;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.lang.String.format;
import static org.globalart.astrum.common.text.UnitFormatter.formatStorage;

/**
 * A {@link BlobStoreQuota} which checks that a blob store has at least a certain amount of space remaining.
 */
@Component
@Qualifier(SpaceRemainingQuota.ID)
public class SpaceRemainingQuota
    extends BlobStoreQuotaSupport
{
  public static final String ID = "spaceRemainingQuota";

  private static final String DISPLAY_NAME = "Space Remaining";

  @Override
  public void validateConfig(final BlobStoreConfiguration config) {
    if (getLimit(config) <= 0) {
      throw new ValidationErrorsException(DISPLAY_NAME + " quotas must have a Quota Limit greater than 0");
    }
  }

  @Override
  public BlobStoreQuotaResult check(final BlobStore blobStore) {
    checkNotNull(blobStore);

    long availableSpace = blobStore.getMetrics().getAvailableSpace();
    boolean isUnlimited = blobStore.getMetrics().isUnlimited();
    long limit = getLimit(blobStore.getBlobStoreConfiguration());

    String rawName = blobStore.getBlobStoreConfiguration().getName();
    String escapedName = StringEscapeUtils.escapeHtml4(rawName);
    String msg = format("Blob store %s is limited to having %s available space, and has %s space remaining",
        escapedName,
        formatStorage(limit),
        formatStorage(availableSpace));

    return new BlobStoreQuotaResult(!isUnlimited && availableSpace < limit, rawName, msg);
  }

  @Override
  public String getDisplayName() {
    return DISPLAY_NAME;
  }

  @Override
  public String getId() {
    return ID;
  }
}
