
package org.globalart.astrum.self.hosted.repository.manager;

import java.util.List;
import java.util.Optional;

import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.jmx.reflect.ManagedObject;
import org.globalart.astrum.repository.Recipe;
import org.globalart.astrum.repository.config.ConfigurationFacet;
import org.globalart.astrum.repository.config.ConfigurationStore;
import org.globalart.astrum.repository.manager.ConfigurationValidator;
import org.globalart.astrum.repository.manager.DefaultRepositoriesContributor;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.manager.internal.BaseRepositoryManager;
import org.globalart.astrum.repository.manager.internal.FailedRepositoryTracker;
import org.globalart.astrum.repository.manager.internal.GroupMemberMappingCache;
import org.globalart.astrum.repository.manager.internal.HttpAuthenticationSecretEncoder;
import org.globalart.astrum.repository.manager.internal.RepositoryAdminSecurityContributor;
import org.globalart.astrum.repository.manager.internal.RepositoryFactory;
import org.globalart.astrum.repository.manager.internal.SecretEncoder;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.REPOSITORIES;

/**
 * {@link BaseRepositoryManager} for self-hosted deployments.
 * No overrides needed, only annotations for DI.
 */
@Component
@ManagedLifecycle(phase = REPOSITORIES)
@ManagedObject(
    domain = "org.globalart.astrum.repository.manager",
    typeClass = RepositoryManager.class,
    description = "Repository manager")
public class RepositoryManagerImpl
    extends BaseRepositoryManager<BlobStoreManager>
{
  @Autowired
  public RepositoryManagerImpl(
      final EventManager eventManager,
      final ConfigurationStore store,
      final RepositoryFactory factory,
      final Provider<ConfigurationFacet> configFacet,
      @Lazy final List<Recipe> recipes,
      final RepositoryAdminSecurityContributor securityContributor,
      final List<DefaultRepositoriesContributor> defaultRepositoriesContributors,
      @Value("${astrum.skipDefaultRepositories:false}") final boolean skipDefaultRepositories,
      final BlobStoreManager blobStoreManager,
      final GroupMemberMappingCache groupMemberMappingCache,
      final List<ConfigurationValidator> configurationValidators,
      final HttpAuthenticationSecretEncoder httpAuthenticationSecretEncoder,
      final FailedRepositoryTracker failedRepositoryTracker,
      final Optional<SecretEncoder> ecrSecretEncoder)
  {
    super(eventManager, store, factory, configFacet, recipes, securityContributor, defaultRepositoriesContributors,
        skipDefaultRepositories, blobStoreManager, groupMemberMappingCache, configurationValidators,
        httpAuthenticationSecretEncoder, failedRepositoryTracker, ecrSecretEncoder.orElse(null));
  }
}
