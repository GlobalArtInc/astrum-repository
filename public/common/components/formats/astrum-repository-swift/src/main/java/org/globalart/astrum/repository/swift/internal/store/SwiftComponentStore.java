package org.globalart.astrum.repository.swift.internal.store;

import java.util.Collection;
import java.util.List;

import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.content.store.ComponentStore;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;

import static org.globalart.astrum.common.app.FeatureFlags.DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE;

public class SwiftComponentStore
    extends ComponentStore<SwiftComponentDAO>
{
  @Autowired
  public SwiftComponentStore(
      final DataSessionSupplier sessionSupplier,
      @Value(DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE) final boolean clustered,
      final String storeName)
  {
    super(sessionSupplier, clustered, storeName, SwiftComponentDAO.class);
  }

  @Transactional
  public void replacePackageUrls(final int repositoryId, final int componentId, final Collection<String> urls) {
    dao().deletePackageUrls(componentId);
    for (String url : urls) {
      dao().addPackageUrl(repositoryId, componentId, url);
    }
  }

  @Transactional
  public List<String> findPackageIdentifiers(final int repositoryId, final String url) {
    return dao().findPackageIdentifiers(repositoryId, url);
  }
}
