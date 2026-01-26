
package org.globalart.astrum.blobstore.quota.internal;

import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.quota.BlobStoreQuota;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaResult;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaService;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Optional.ofNullable;
import static org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport.ROOT_KEY;
import static org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport.TYPE_KEY;
import org.springframework.stereotype.Component;

/**
 * Default implementation of {@link BlobStoreQuotaService}
 *
 * @since 3.14
 */
@Component
public class BlobStoreQuotaServiceImpl
    implements BlobStoreQuotaService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Map<String, BlobStoreQuota> quotas;

  @Autowired
  public BlobStoreQuotaServiceImpl(final List<BlobStoreQuota> quotasList) {
    this.quotas = QualifierUtil.buildQualifierBeanMap(checkNotNull(quotasList));
  }

  @Override
  public void validateSoftQuotaConfig(final BlobStoreConfiguration config) {
    getQuotaType(config).ifPresent(type -> {
      if (!quotas.containsKey(type)) {
        throw new ValidationErrorsException("To enable Soft Quota, you must select a Type of Quota");
      }
    });
    getQuota(config).ifPresent(quota -> quota.validateConfig(config));
  }

  private Optional<String> getQuotaType(final BlobStoreConfiguration config) {
    return ofNullable(config.attributes(ROOT_KEY).get(TYPE_KEY, String.class));
  }

  private Optional<BlobStoreQuota> getQuota(final BlobStoreConfiguration config) {
    Optional<String> quotaType = getQuotaType(config);
    Optional<BlobStoreQuota> quota = quotaType.map(quotas::get);

    if (quotaType.isPresent() && !quota.isPresent()) {
      log.error("For blob store {} unable to find quota type for key {}", config.getName(), quotaType.get());
    }
    return quota;
  }

  @Nullable
  @Override
  public BlobStoreQuotaResult checkQuota(final BlobStore blobStore) {
    checkNotNull(blobStore);
    BlobStoreConfiguration config = blobStore.getBlobStoreConfiguration();

    return getQuota(config).map(quota -> {
      log.debug("Checking blob store {} for quota {}", config.getName(), quota);
      return quota.check(blobStore);
    }).orElse(null);
  }
}
