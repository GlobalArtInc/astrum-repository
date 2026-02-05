
package org.globalart.astrum.repository.content.facet;

import java.time.OffsetDateTime;
import java.util.HashSet;
import java.util.Optional;
import java.util.Set;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.IllegalOperationException;
import org.globalart.astrum.repository.RedeployDisabledException;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationFacet;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AttributeOperation;
import org.globalart.astrum.repository.content.ContentRepository;
import org.globalart.astrum.repository.content.fluent.FluentAssets;
import org.globalart.astrum.repository.content.fluent.FluentBlobs;
import org.globalart.astrum.repository.content.fluent.FluentComponents;
import org.globalart.astrum.repository.content.fluent.internal.FluentAssetsImpl;
import org.globalart.astrum.repository.content.fluent.internal.FluentBlobsImpl;
import org.globalart.astrum.repository.content.fluent.internal.FluentComponentsImpl;
import org.globalart.astrum.repository.content.store.ContentRepositoryData;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.search.normalize.VersionNormalizerService;
import org.globalart.astrum.repository.storage.BlobMetadataStorage;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.security.ClientInfo;
import org.globalart.astrum.transaction.Transactional;
import org.globalart.astrum.transaction.TransactionalStore;

import com.google.common.annotations.VisibleForTesting;
import jakarta.validation.ConstraintViolation;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.groups.Default;
import org.springframework.beans.factory.annotation.Autowired;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;
import static java.lang.String.format;
import static java.util.Optional.ofNullable;
import static org.globalart.astrum.blobstore.api.BlobStore.CONTENT_TYPE_HEADER;
import static org.globalart.astrum.blobstore.api.BlobStoreManager.DEFAULT_BLOBSTORE_NAME;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;
import static org.globalart.astrum.repository.config.WritePolicy.ALLOW;
import static org.globalart.astrum.validation.ConstraintViolations.maybeAdd;
import static org.globalart.astrum.validation.ConstraintViolations.maybePropagate;

/**
 * {@link ContentFacet} support.
 *
 * @since 3.24
 */
public abstract class ContentFacetSupport
    extends FacetSupport
    implements ContentFacet, TransactionalStore<DataSession<?>>
{
  private final FormatStoreManager formatStoreManager;

  @VisibleForTesting
  static class Config
  {
    @NotEmpty
    public String blobStoreName = DEFAULT_BLOBSTORE_NAME;

    @NotEmpty
    public String dataStoreName = DEFAULT_DATASTORE_NAME;

    @NotNull(groups = HostedType.ValidationGroup.class)
    public WritePolicy writePolicy;

    @NotNull
    public Boolean strictContentTypeValidation = Boolean.TRUE;

    @Override
    public String toString() {
      return getClass().getSimpleName() + "{" +
          "blobStoreName='" + blobStoreName + '\'' +
          ", dataStoreName='" + dataStoreName + '\'' +
          ", writePolicy=" + writePolicy +
          ", strictContentTypeValidation=" + strictContentTypeValidation +
          '}';
    }
  }

  private ContentFacetDependencies dependencies;

  private Config config;

  private ContentFacetStores stores;

  private FluentBlobs fluentBlobs;

  private FluentComponents fluentComponents;

  private FluentAssets fluentAssets;

  private EntityId configRepositoryId;

  private Integer contentRepositoryId;

  private AssetBlobValidator assetBlobValidator;

  protected ContentFacetSupport(final FormatStoreManager formatStoreManager) {
    this.formatStoreManager = checkNotNull(formatStoreManager);
  }

  @Autowired
  @VisibleForTesting
  protected final void setDependencies(final ContentFacetDependencies dependencies) {
    this.dependencies = checkNotNull(dependencies);
  }

  @Override
  protected void doValidate(final Configuration configuration) throws Exception {
    facet(ConfigurationFacet.class).validateSection(configuration, STORAGE, Config.class,
        Default.class, getRepository().getType().getValidationGroup());

    Config configToValidate = facet(ConfigurationFacet.class).readSection(configuration, STORAGE, Config.class);

    Set<ConstraintViolation<?>> violations = new HashSet<>();
    // Only validate blob store group membership if the blob store exists
    // Existence validation happens in BaseRepositoryManager.validateConfiguration() during create/update
    maybeAdd(violations, validateBlobStoreNotInGroup(configToValidate.blobStoreName));
    maybePropagate(violations, log);
  }

  private ConstraintViolation<?> validateBlobStoreNotInGroup(final String blobStoreName) {
    // Skip validation if blob store doesn't exist - allow repositories to load even if their blob store
    // is missing (e.g., deleted, corrupted DB) so the application can start and the repository can be fixed/deleted
    // rather than blocking entire application startup
    BlobStore blobStore = dependencies.getBlobStoreManager().get(blobStoreName);
    if (blobStore == null) {
      return null;
    }

    return dependencies.getBlobStoreManager()
        .getParent(blobStoreName)
        .map(groupName -> dependencies.getConstraintViolationFactory()
            .createViolation(format("%s.blobStoreName", STORAGE),
                format("Blob Store '%s' is a member of Blob Store Group '%s' and cannot be set as storage",
                    blobStoreName, groupName)))
        .orElse(null);
  }

  @Override
  protected void doConfigure(final Configuration configuration) throws Exception {
    config = facet(ConfigurationFacet.class).readSection(configuration, STORAGE, Config.class);
    log.debug("Config: {}", config);

    stores = new ContentFacetStores(
        dependencies.getBlobStoreManager(), config.blobStoreName,
        formatStoreManager, config.dataStoreName);

    fluentBlobs = new FluentBlobsImpl(this, stores.blobStoreProvider);
    fluentComponents = new FluentComponentsImpl(this, stores.componentStore,
        version -> versionNormalizerService().getNormalizedVersionByFormat(version, getRepository().getFormat()));
    fluentAssets = new FluentAssetsImpl(this, stores.assetStore);
  }

  @Override
  protected void doStart() throws Exception {
    // configuration is persisted by this point and has been assigned an id
    configRepositoryId = getRepository().getConfiguration().getRepositoryId();

    checkState(configRepositoryId != null, "Missing configRepositoryId");

    // get or create the associated content repository
    contentRepositoryId = stores.contentRepositoryStore.readContentRepository(configRepositoryId)
        .orElseGet(this::createContentRepository)
        .contentRepositoryId();

    checkState(contentRepositoryId != null, "Missing contentRepositoryId");

    assetBlobValidator = dependencies.getAssetBlobValidators().selectValidator(getRepository());
  }

  @Override
  protected void doDestroy() throws Exception {
    config = null;
  }

  /**
   * Creates a {@link ContentRepository} for the associated config repository.
   */
  private final ContentRepository createContentRepository() {
    ContentRepositoryData contentRepository = newContentRepository();
    contentRepository.setConfigRepositoryId(configRepositoryId);
    stores.contentRepositoryStore.createContentRepository(contentRepository);
    return contentRepository;
  }

  /**
   * Retrieves the latest {@link ContentRepository} data from the store.
   */
  protected final ContentRepository contentRepository() {
    return stores.contentRepositoryStore
        .readContentRepository(configRepositoryId)
        .orElseThrow(() -> new IllegalStateException("Missing content repository for " + configRepositoryId));
  }

  @Override
  @Transactional
  protected void doDelete() throws Exception {
    if (configRepositoryId != null) {
      stores.assetStore.deleteAssets(contentRepositoryId);
      stores.componentStore.deleteComponents(contentRepositoryId);
      stores.contentRepositoryStore.deleteContentRepository(configRepositoryId);
    }
  }

  @Override
  public final EntityId configRepositoryId() {
    return configRepositoryId;
  }

  @Override
  public final Integer contentRepositoryId() {
    return contentRepositoryId;
  }

  @Override
  public final NestedAttributesMap attributes() {
    return contentRepository().attributes();
  }

  @Override
  public final OffsetDateTime created() {
    return contentRepository().created();
  }

  @Override
  public final OffsetDateTime lastUpdated() {
    return contentRepository().lastUpdated();
  }

  @Override
  public final ContentFacet attributes(final AttributeOperation change, final String key, final Object value) {
    stores.contentRepositoryStore.updateContentRepositoryAttributes(contentRepository(), change, key, value);
    return this;
  }

  @Override
  public final FluentBlobs blobs() {
    return fluentBlobs;
  }

  @Override
  public final FluentComponents components() {
    return fluentComponents;
  }

  @Override
  public final FluentAssets assets() {
    return fluentAssets;
  }

  public ContentFacetDependencies dependencies() {
    return dependencies;
  }

  @Override
  public final DataSession<?> openSession() {
    return dependencies.getDataSessionSupplier().openSession(config.dataStoreName);
  }

  public final Optional<ClientInfo> clientInfo() {
    return ofNullable(dependencies.getClientInfoProvider().getCurrentThreadClientInfo());
  }

  public final String nodeName() {
    return dependencies.getNodeAccess().getId();
  }

  public final BlobMetadataStorage blobMetadataStorage() {
    return dependencies.getBlobMetadataStorage();
  }

  public final VersionNormalizerService versionNormalizerService() {
    return dependencies.getVersionNormalizerService();
  }

  public final Repository repository() {
    return getRepository();
  }

  public final ContentFacetStores stores() {
    return stores;
  }

  public final void checkAttachAllowed(final Asset asset) {
    if (!asset.hasBlob()) {
      if (!writePolicy(asset).checkCreateAllowed()) {
        throwNotAllowed(asset, " is read-only");
      }
    }
    else {
      WritePolicy policy = writePolicy(asset);
      if (!policy.checkUpdateAllowed()) {
        // Enhance error message when ALLOW_ONCE prevents update
        String reason = policy == WritePolicy.ALLOW_ONCE
            ? " cannot be updated as asset already exists and redeploy is not allowed"
            : " cannot be updated";
        throw new RedeployDisabledException(getRepository(), asset.path(), reason);
      }
    }
  }

  public final void checkDeleteAllowed(final Asset asset) {
    if (asset.hasBlob() && !writePolicy(asset).checkDeleteAllowed()) {
      throwNotAllowed(asset, " cannot be deleted");
    }
  }

  private void throwNotAllowed(final Asset asset, final String reason) {
    throw new IllegalOperationException(repository().getName() + asset.path() + reason);
  }

  public final String checkContentType(final Asset asset, final Blob blob) {
    return determineContentType(asset, blob, config.strictContentTypeValidation);
  }

  /**
   * Override this method to customize the write-policy of selected assets.
   */
  protected WritePolicy writePolicy(final Asset asset) {
    return getConfiguredWritePolicy();
  }

  protected WritePolicy getConfiguredWritePolicy() {
    return ofNullable(config).map(config -> config.writePolicy).orElse(ALLOW);
  }

  /**
   * Override this method to customize Content-Type validation of asset blobs.
   */
  protected String determineContentType(final Asset asset, final Blob blob, final boolean strictValidation) {
    String contentType = blob.getHeaders().get(CONTENT_TYPE_HEADER);
    return assetBlobValidator.determineContentType(strictValidation, blob::getInputStream, asset.path(), contentType);
  }

  /**
   * Override this method to customize creation of new content repositories.
   */
  protected ContentRepositoryData newContentRepository() {
    return new ContentRepositoryData();
  }
}
