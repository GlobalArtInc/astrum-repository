
package org.globalart.astrum.blobstore.api;

import java.util.List;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.blobstore.api.tasks.BlobStoreConsumer;
import org.globalart.astrum.blobstore.api.tasks.BlobStoreTaskService;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Component
public class BlobStoreTaskServiceImpl
    implements BlobStoreTaskService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  final List<BlobStoreConsumer> blobStoreConsumers;

  @Autowired
  public BlobStoreTaskServiceImpl(final List<BlobStoreConsumer> blobStoreConsumers) {
    this.blobStoreConsumers = blobStoreConsumers;
  }

  @Override
  public boolean isAnyTaskInUseForBlobStore(final String blobStoreName) {
    return blobStoreConsumers.stream().anyMatch(blobStoreConsumer -> blobStoreConsumer.isBlobStoreInUse(blobStoreName));
  }

  @Override
  public int countTasksInUseForBlobStore(final String blobStoreName) {
    return blobStoreConsumers.stream()
        .map(blobStoreConsumer -> blobStoreConsumer.blobStoreUsageCount(blobStoreName))
        .mapToInt(Integer::intValue)
        .sum();
  }
}
