
package org.globalart.astrum.blobstore.group.internal;

import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.group.BlobStoreGroup;
import org.globalart.astrum.blobstore.group.FillPolicy;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * {@link FillPolicy} that writes to first blobstore in group.
 *
 * @since 3.14
 */
@Component
@Qualifier(WriteToFirstMemberFillPolicy.TYPE)
public class WriteToFirstMemberFillPolicy
    implements FillPolicy
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String TYPE = "writeToFirst";

  private static final String NAME = "Write to First";

  @Override
  public String getName() {
    return NAME;
  }

  @Override
  @Nullable
  public BlobStore chooseBlobStore(final BlobStoreGroup blobStoreGroup, final Map<String, String> headers) {
    return blobStoreGroup
        .getMembers()
        .stream()
        .filter(BlobStore::isWritable)
        .filter(BlobStore::isStorageAvailable)
        .findFirst()
        .orElse(null);
  }
}
