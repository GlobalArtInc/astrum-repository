
package org.globalart.astrum.blobstore.external;

import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.stream.Stream;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.common.entity.Continuations;
import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.logging.task.ProgressLogIntervalHelper;
import org.globalart.astrum.logging.task.TaskLogging;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryParallelTaskSupport;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.store.AssetBlobStore;
import org.globalart.astrum.scheduling.Cancelable;
import org.globalart.astrum.scheduling.CancelableHelper;

import com.google.common.base.Predicate;
import com.google.common.base.Predicates;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.blobstore.group.BlobStoreGroup.CONFIG_KEY;
import static org.globalart.astrum.blobstore.group.BlobStoreGroup.MEMBERS_KEY;
import static org.globalart.astrum.logging.task.TaskLogType.TASK_LOG_ONLY;
import static org.globalart.astrum.repository.config.ConfigurationConstants.BLOB_STORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;

/**
 * Retrieves cloud blob store etag and last-modified headers for use with pre-signed URLs.
 */
@Component
@TaskLogging(TASK_LOG_ONLY)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class ExternalMetadataTask
    extends RepositoryParallelTaskSupport
    implements Cancelable
{
  public static final String FORMAT_FIELD_ID = "external.metadata.repository.format";

  // Type strings are hardcoded — their constants live in private/ modules inaccessible from public/
  // S3_TYPE matches S3BlobStore.TYPE; AZURE_TYPE matches AzureBlobStore.TYPE (azure-cloud module)
  private static final String S3_TYPE = "S3";

  private static final String AZURE_TYPE = "Azure Cloud Storage";

  private final BlobStoreManager blobStoreManager;

  private final AtomicInteger processed = new AtomicInteger();

  /**
   * @param blobStoreManager the blob store manager
   * @param concurrencyLimit the number of concurrent threads processing the queue allowed.
   */
  @Autowired
  public ExternalMetadataTask(
      final BlobStoreManager blobStoreManager,
      @Value("${external.metadata.repository.concurrencyLimit:5}") final int concurrencyLimit)
  {
    super(concurrencyLimit);
    this.blobStoreManager = checkNotNull(blobStoreManager);
  }

  @Override
  public String getMessage() {
    return String.format("retrieving remote blob headers for repositories '%s'", getRepositoryField());
  }

  @Override
  protected Object result() {
    return processed.get();
  }

  @Override
  protected Stream<Runnable> jobStream(final ProgressLogIntervalHelper progress, final Repository repository) {
    log.info("Processing {}", repository.getName());

    BlobStore blobstore = getBlobStore(repository).orElse(null);
    if (blobstore == null) {
      log.error("Unable to obtain blobstore for {}", repository);
      return Stream.of();
    }

    AssetBlobStore<?> assetBlobStore =
        ((ContentFacetSupport) repository.facet(ContentFacet.class)).stores().assetBlobStore();

    return Continuations.streamOf(repository.facet(ContentFacet.class).assets()::browse)
        .filter(FluentAsset::hasBlob)
        .map(FluentAsset::blob)
        .map(Optional::get)
        .filter(assetBlob -> assetBlob.externalMetadata() == null)
        .map(assetBlob -> createJob(progress, assetBlobStore, blobstore, assetBlob));
  }

  private Runnable createJob(
      final ProgressLogIntervalHelper progress,
      final AssetBlobStore<?> assetBlobStore,
      final BlobStore blobstore,
      final AssetBlob assetBlob)
  {
    return () -> {
      // check cancellation to quickly drain the queue
      CancelableHelper.checkCancellation();
      log.debug("Processing {}", assetBlob);

      blobstore.getExternalMetadata(assetBlob.blobRef())
          .ifPresent(remoteMetadata -> assetBlobStore.setExternalMetadata(assetBlob, remoteMetadata));

      processed.addAndGet(1);
      progress.info("Processed {} assets", processed.get());
    };
  }

  @Override
  protected boolean appliesTo(final Repository repository) {
    if (!formatMatcher().test(repository)) {
      return false;
    }

    Optional<BlobStore> blobstore = getBlobStore(repository);

    if (blobstore.isEmpty()) {
      log.error("Unable to identify blobstore type for repository {}", repository);
      return false;
    }

    if (supportsMetadata(blobstore.get())) {
      return true;
    }
    else if (!isGroup(blobstore.get())) {
      return false;
    }

    return getMembers(blobstore.get()).stream()
        .map(blobStoreManager::get)
        .anyMatch(ExternalMetadataTask::supportsMetadata);
  }

  private Optional<BlobStore> getBlobStore(final Repository repository) {
    return Optional.ofNullable(repository.getConfiguration())
        .map(Configuration::getAttributes)
        .map(attr -> attr.get(STORAGE))
        .filter(Map.class::isInstance)
        .map(Map.class::cast)
        .map(storage -> storage.get(BLOB_STORE_NAME))
        .filter(Objects::nonNull)
        .filter(String.class::isInstance)
        .map(String.class::cast)
        .map(blobStoreManager::get);
  }

  protected Predicate<Repository> formatMatcher() {
    String format = Optional.ofNullable(getConfiguration().getString(FORMAT_FIELD_ID))
        .map(String::trim)
        .orElse(null);
    if (Strings2.isBlank(format) || format.equals("*")) {
      return Predicates.alwaysTrue();
    }
    return repository -> repository.getFormat().getValue().equals(format);
  }

  private static boolean supportsMetadata(final BlobStore blobStore) {
    String type = getType(blobStore);
    return S3_TYPE.equals(type) || AZURE_TYPE.equals(type);
  }

  private static boolean isGroup(final BlobStore blobStore) {
    return "Group".equals(getType(blobStore));
  }

  private static String getType(final BlobStore blobStore) {
    return blobStore.getBlobStoreConfiguration().getType();
  }

  @SuppressWarnings("unchecked")
  private static List<String> getMembers(final BlobStore blobstore) {
    return blobstore.getBlobStoreConfiguration().attributes(CONFIG_KEY).get(MEMBERS_KEY, List.class);
  }
}
