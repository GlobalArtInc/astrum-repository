
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
 * A {@link BlobStoreQuota} which checks that a blob store isn't using more space than the limit.
 */
@Component
@Qualifier(SpaceUsedQuota.ID)
public class SpaceUsedQuota
    extends BlobStoreQuotaSupport
{
  public static final String ID = "spaceUsedQuota";

  private static final String DISPLAY_NAME = "Space Used";

  @Override
  public void validateConfig(final BlobStoreConfiguration config) {
    if (getLimit(config) <= 0) {
      throw new ValidationErrorsException(DISPLAY_NAME + " quotas must have a Quota Limit greater than 0");
    }
  }

  @Override
  public BlobStoreQuotaResult check(final BlobStore blobStore) {
    checkNotNull(blobStore);

    long usedSpace = blobStore.getMetrics().getTotalSize();
    long limit = getLimit(blobStore.getBlobStoreConfiguration());

    String rawName = blobStore.getBlobStoreConfiguration().getName();
    String escapedName = StringEscapeUtils.escapeHtml4(rawName);
    String msg = format("Blob store %s is using %s space and has a limit of %s", escapedName,
        formatStorage(usedSpace),
        formatStorage(limit));

    return new BlobStoreQuotaResult(usedSpace > limit, rawName, msg);
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
