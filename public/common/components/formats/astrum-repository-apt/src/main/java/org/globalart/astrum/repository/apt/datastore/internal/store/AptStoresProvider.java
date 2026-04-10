
package org.globalart.astrum.repository.apt.datastore.internal.store;

import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.apt.datastore.internal.browse.AptBrowseNodeDAO;
import org.globalart.astrum.repository.content.browse.store.BrowseNodeStore;
import org.globalart.astrum.repository.content.store.AssetBlobStore;
import org.globalart.astrum.repository.content.store.BespokeFormatStoreSupport;
import org.globalart.astrum.repository.content.store.ComponentStore;
import org.globalart.astrum.repository.content.store.ContentRepositoryStore;

import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Configures the content store bindings for a Apt format.
 */
@Qualifier(AptFormat.NAME)
public class AptStoresProvider
    extends
    BespokeFormatStoreSupport<ContentRepositoryStore<AptContentRepositoryDAO>, ComponentStore<AptComponentDAO>, AptAssetStore, AssetBlobStore<AptAssetBlobDAO>, BrowseNodeStore<AptBrowseNodeDAO>>
{
  // nothing to add
}
