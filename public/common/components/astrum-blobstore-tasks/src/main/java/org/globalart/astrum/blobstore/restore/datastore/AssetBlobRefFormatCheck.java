
package org.globalart.astrum.blobstore.restore.datastore;

import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.store.AssetBlobStore;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Optional.ofNullable;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.DATA_STORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;
import org.springframework.stereotype.Component;

/**
 * Utility class which checks whether asset blob refs no longer contain node ids.
 *
 */
@Component
public class AssetBlobRefFormatCheck
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Map<String, FormatStoreManager> formatStoreManagers;

  @Autowired
  public AssetBlobRefFormatCheck(final List<FormatStoreManager> formatStoreManagersList) {
    this.formatStoreManagers = QualifierUtil.buildQualifierBeanMap(checkNotNull(formatStoreManagersList));
  }

  /**
   * Returns true if the specified repository contains node id in its asset blob refs,
   * otherwise returns false
   */
  public boolean isAssetBlobRefNotMigrated(final Repository repository) {
    final String dataStoreName = getDataStoreName(repository);
    final String format = repository.getFormat().getValue();
    final boolean notMigrated = ofNullable(formatStoreManagers.get(format))
        .map(storeManager -> storeManager.assetBlobStore(dataStoreName))
        .map(assetBlobStore -> ((AssetBlobStore<?>) assetBlobStore).notMigratedAssetBlobRefsExists())
        .orElseThrow(() -> new RuntimeException("Cannot determine asset blob ref migration status"));

    if (notMigrated) {
      log.warn("Cannot restore {} repository '{}' " + "because legacy blob ref migration is not complete.",
          repository.getFormat().getValue(), repository.getName());
    }

    return notMigrated;
  }

  private String getDataStoreName(final Repository repository) {
    return Optional.of(repository.getConfiguration())
        .map(configuration -> configuration.attributes(STORAGE))
        .map(attr -> attr.get(DATA_STORE_NAME, String.class))
        .orElse(DEFAULT_DATASTORE_NAME);
  }
}
