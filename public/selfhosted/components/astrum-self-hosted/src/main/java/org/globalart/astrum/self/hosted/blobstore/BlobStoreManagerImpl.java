
package org.globalart.astrum.self.hosted.blobstore;

import java.util.List;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import org.globalart.astrum.blobstore.BlobStoreDescriptor;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.DefaultBlobStoreProvider;
import org.globalart.astrum.blobstore.api.tasks.BlobStoreTaskService;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.app.ManagedLifecycle.Phase;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.jmx.reflect.ManagedObject;
import org.globalart.astrum.repository.blobstore.BlobStoreConfigurationStore;
import org.globalart.astrum.repository.internal.blobstore.BaseBlobStoreManager;
import org.globalart.astrum.repository.internal.blobstore.BlobStoreOverride;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.replication.ReplicationBlobStoreStatusManager;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

/**
 * The {@link org.globalart.astrum.blobstore.api.BlobStoreManager} implementation for Self-Hosted deployments.
 *
 * No overrides needed from the {@link BaseBlobStoreManager}, just annotations needed for DI.
 */
@Component
@ManagedObject
@ManagedLifecycle(phase = Phase.STORAGE)
public class BlobStoreManagerImpl
    extends BaseBlobStoreManager
{
  @Autowired
  public BlobStoreManagerImpl(
      final EventManager eventManager, // NOSONAR
      final BlobStoreConfigurationStore store,
      final @Lazy List<BlobStoreDescriptor> blobStoreDescriptors,
      final List<Provider<BlobStore>> blobStorePrototypes,
      final @Lazy Provider<RepositoryManager> repositoryManagerProvider,
      final NodeAccess nodeAccess,
      @Nullable @Value("${astrum.blobstore.provisionDefaults:#{null}}") final Boolean provisionDefaults,
      @Nullable final DefaultBlobStoreProvider blobstoreProvider,
      final BlobStoreTaskService blobStoreTaskService,
      final Provider<BlobStoreOverride> blobStoreOverrideProvider,
      final ReplicationBlobStoreStatusManager replicationBlobStoreStatusManager,
      final SecretsService secretService)
  {
    super(eventManager, store, blobStoreDescriptors, blobStorePrototypes,
        repositoryManagerProvider, nodeAccess, provisionDefaults, blobstoreProvider, blobStoreTaskService,
        blobStoreOverrideProvider, replicationBlobStoreStatusManager, secretService);
  }
}
