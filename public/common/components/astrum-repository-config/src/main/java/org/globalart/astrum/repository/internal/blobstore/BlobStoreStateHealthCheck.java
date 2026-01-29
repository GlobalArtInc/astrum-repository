
package org.globalart.astrum.repository.internal.blobstore;

import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;
import java.util.stream.StreamSupport;

import jakarta.inject.Provider;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.blobstore.group.BlobStoreGroup;

import com.codahale.metrics.health.HealthCheck;
import com.google.common.base.Preconditions;
import com.google.common.collect.Iterators;
import org.apache.commons.text.StringEscapeUtils;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static java.lang.String.format;

/**
 * Inform on the health of all BlobStores based on their lifecycle state.
 */
@Component
@Qualifier("Blob Stores Ready")
public class BlobStoreStateHealthCheck
    extends HealthCheck
{
  private final Provider<BlobStoreManager> blobStoreManagerProvider;

  @Autowired
  public BlobStoreStateHealthCheck(final Provider<BlobStoreManager> blobStoreManagerProvider) {
    this.blobStoreManagerProvider = Preconditions.checkNotNull(blobStoreManagerProvider);
  }

  @Override
  protected Result check() {
    final List<String> blobStores = StreamSupport.stream(blobStoreManagerProvider.get().browse().spliterator(), false)
        .map(blobStore -> {
          final String name = StringEscapeUtils.escapeHtml4(blobStore.getBlobStoreConfiguration().getName());
          if (!blobStore.isStarted()) {
            return format("Blob store '%s' reports as not started", name);
          }
          if (!blobStore.getBlobStoreConfiguration().getType().equals(BlobStoreGroup.TYPE) &&
              !blobStore.isWritable()) {
            return format("Blob store '%s' reports as not writeable", name);
          }
          if (!blobStore.isStorageAvailable()) {
            return format("Blob store '%s' reports as not available", name);
          }
          return null;
        })
        .filter(Objects::nonNull)
        .collect(Collectors.toList());

    String message = format("%s/%s blob stores report issues<br>%s", blobStores.size(),
        Iterators.size(blobStoreManagerProvider.get().browse().iterator()),
        String.join("<br>", blobStores));

    return blobStores.isEmpty() ? Result.healthy(message) : Result.unhealthy(message);
  }
}
