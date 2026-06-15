
package org.globalart.astrum.coreui.internal.blobstore;

import java.util.List;
import java.util.Objects;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import com.google.common.base.Predicates;
import com.google.common.base.Strings;

import org.globalart.astrum.blobstore.BlobStoreDescriptor;
import org.globalart.astrum.blobstore.BlobStoreDescriptorProvider;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.blobstore.api.BlobStoreMetrics;
import org.globalart.astrum.blobstore.group.BlobStoreGroup;
import org.globalart.astrum.blobstore.quota.BlobStoreQuota;
import org.globalart.astrum.blobstore.file.FileBlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationHelper;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.blobstore.BlobStoreConfigurationStore;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.rest.Resource;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.coreui.internal.blobstore.BlobStoreInternalResource.RESOURCE_PATH;
import org.springframework.stereotype.Component;

/**
 * @since 3.30
 */
@Component
@Path(RESOURCE_PATH)
public class BlobStoreInternalResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  static final String RESOURCE_PATH = "/internal/ui/blobstores";

  public static final String GOOGLE_CONFIG = "google cloud storage";

  public static final String GOOGLE_TYPE = "google";

  public static final String GOOGLE_BUCKET_KEY = "bucketName";

  public static final String PREFIX_KEY = "prefix";

  public static final String AZURE_CONFIG = "azure cloud storage";

  public static final String AZURE_TYPE = "azure";

  public static final String CONTAINER_NAME = "containerName";

  private final BlobStoreManager blobStoreManager;

  private final BlobStoreConfigurationStore store;

  private final BlobStoreDescriptorProvider blobStoreDescriptorProvider;

  private final List<BlobStoreQuotaTypesUIResponse> blobStoreQuotaTypes;

  private final RepositoryManager repositoryManager;

  private static final Logger logger = LoggerFactory.getLogger(BlobStoreInternalResource.class);

  @Autowired
  public BlobStoreInternalResource(
      final BlobStoreManager blobStoreManager,
      final BlobStoreConfigurationStore store,
      final BlobStoreDescriptorProvider blobStoreDescriptorProvider,
      final List<BlobStoreQuota> quotaFactoriesList,
      final RepositoryManager repositoryManager)
  {
    this.blobStoreManager = checkNotNull(blobStoreManager);
    this.store = checkNotNull(store);
    this.blobStoreDescriptorProvider = checkNotNull(blobStoreDescriptorProvider);
    this.blobStoreQuotaTypes = QualifierUtil.buildQualifierBeanMap(quotaFactoriesList)
        .entrySet()
        .stream()
        .map(BlobStoreQuotaTypesUIResponse::new)
        .toList();
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @RequiresAuthentication
  @RequiresPermissions("astrum:blobstores:read")
  @GET
  public List<BlobStoreUIResponse> listBlobStores() {

    return store.list()
        .stream()
        .map(configuration -> {
          String blobStoreType = configuration.getType();
          BlobStoreDescriptor blobStoreDescriptor = Optional.ofNullable(blobStoreDescriptorProvider.get())
              .map(it -> it.get(blobStoreType))
              .orElse(null);
          if (blobStoreDescriptor == null) {
            return null;
          }
          String typeId = blobStoreDescriptor.getId();

          final String path = getPath(typeId.toLowerCase(), configuration);
          BlobStoreMetrics metrics = Optional.ofNullable(blobStoreManager.get(configuration.getName()))
              .map(BlobStoreInternalResource::getBlobStoreMetrics)
              .orElse(null);
          return new BlobStoreUIResponse(typeId, configuration, metrics, path);
        })
        .filter(Objects::nonNull)
        .toList();
  }

  // If a blobstore hasn't started due to an error we still want to return it from the api.
  // To achieve this, we use a null metrics object which will show the BlobStore as unavailable.
  private static BlobStoreMetrics getBlobStoreMetrics(final BlobStore bs) {
    if (bs.isGroupable()) {
      return bs.isStarted() ? bs.getMetrics() : null;
    }
    else {
      return ((BlobStoreGroup) bs).getMembers()
          .stream()
          .map(BlobStore::isStarted)
          .reduce(Boolean::logicalAnd)
          .orElse(false) ? bs.getMetrics() : null;
    }
  }

  private static String getPath(final String typeId, BlobStoreConfiguration configuration) {
    if (typeId.equals(FileBlobStore.TYPE.toLowerCase())) {
      return getFilePath(configuration);
    }
    else if (typeId.equals(S3BlobStoreConfigurationHelper.CONFIG_KEY)) {
      return getS3Path(configuration);
    }
    else if (typeId.equals(AZURE_TYPE)) {
      return getAzurePath(configuration);
    }
    else if (typeId.equals(BlobStoreGroup.TYPE.toLowerCase())) {
      return "N/A";
    }
    else if (typeId.equals(GOOGLE_TYPE)) {
      return getGooglePath(configuration);
    }
    logger.warn("blob store type {} unknown, defaulting to N/A for path", typeId);
    return "N/A";
  }

  private static String getFilePath(BlobStoreConfiguration configuration) {
    try {
      String path = configuration.attributes(FileBlobStore.CONFIG_KEY).get(FileBlobStore.PATH_KEY, String.class);
      return Strings.isNullOrEmpty(path) ? "N/A" : path;
    }
    catch (NullPointerException | ClassCastException e) {
      logger.warn("Error getting file path from configuration, using N/A", e);
      return "N/A";
    }
  }

  private static String getS3Path(BlobStoreConfiguration configuration) {
    try {
      String bucket = configuration
          .attributes(S3BlobStoreConfigurationHelper.CONFIG_KEY)
          .get(S3BlobStoreConfigurationHelper.BUCKET_KEY, String.class);

      if (Strings.isNullOrEmpty(bucket)) {
        return "N/A";
      }

      String prefix = S3BlobStoreConfigurationHelper.getBucketPrefix(configuration);
      if (Strings.isNullOrEmpty(prefix)) {
        return bucket;
      }

      String cleanPrefix = prefix.replaceFirst("/$", "");
      return bucket + "/" + cleanPrefix;
    }
    catch (NullPointerException | ClassCastException e) {
      logger.warn("Error getting S3 bucket path from configuration, using N/A", e);
      return "N/A";
    }
  }

  private static String getAzurePath(BlobStoreConfiguration configuration) {
    try {
      String containerName = configuration.attributes(AZURE_CONFIG).get(CONTAINER_NAME, String.class);
      return Strings.isNullOrEmpty(containerName) ? "N/A" : containerName;
    }
    catch (NullPointerException | ClassCastException e) {
      logger.warn("Error getting Azure container path from configuration, using N/A", e);
      return "N/A";
    }
  }

  private static String getGooglePath(BlobStoreConfiguration configuration) {
    try {
      String bucketName = configuration.attributes(GOOGLE_CONFIG).get(GOOGLE_BUCKET_KEY, String.class);
      if (Strings.isNullOrEmpty(bucketName)) {
        return "N/A";
      }

      String prefix = Optional.ofNullable(configuration.attributes(GOOGLE_CONFIG).get(PREFIX_KEY, String.class))
          .filter(Predicates.not(Strings::isNullOrEmpty))
          .map(s -> s.replaceFirst("/$", ""))
          .orElse("");

      return Strings.isNullOrEmpty(prefix) ? bucketName : bucketName + "/" + prefix;
    }
    catch (NullPointerException | ClassCastException e) {
      logger.warn("Error getting Google bucket path from configuration, using N/A", e);
      return "N/A";
    }
  }

  @RequiresAuthentication
  @RequiresPermissions("astrum:blobstores:read")
  @GET
  @Path("/types")
  public List<BlobStoreTypesUIResponse> listBlobStoreTypes() {
    return blobStoreDescriptorProvider.get()
        .entrySet()
        .stream()
        .filter(entry -> entry.getValue().isEnabled())
        .map(BlobStoreTypesUIResponse::new)
        .toList();
  }

  @RequiresAuthentication
  @RequiresPermissions("astrum:blobstores:read")
  @GET
  @Path("/usage/{name}")
  public BlobStoreUsageUIResponse getBlobStoreUsage(@PathParam("name") final String name) {
    long repositoryUsage = repositoryManager.blobstoreUsageCount(name);
    long blobStoreUsage = blobStoreManager.blobStoreUsageCount(name);

    return new BlobStoreUsageUIResponse(repositoryUsage, blobStoreUsage);
  }

  @RequiresAuthentication
  @RequiresPermissions("astrum:blobstores:read")
  @GET
  @Path("/quotaTypes")
  public List<BlobStoreQuotaTypesUIResponse> listQuotaTypes() {
    return blobStoreQuotaTypes;
  }
}
