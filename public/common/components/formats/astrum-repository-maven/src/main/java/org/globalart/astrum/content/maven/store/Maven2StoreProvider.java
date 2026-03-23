
package org.globalart.astrum.content.maven.store;

import org.globalart.astrum.content.maven.internal.browse.Maven2BrowseNodeDAO;
import org.globalart.astrum.repository.content.browse.store.BrowseNodeStore;
import org.globalart.astrum.repository.content.store.AssetBlobStore;
import org.globalart.astrum.repository.content.store.BespokeFormatStoreSupport;
import org.globalart.astrum.repository.content.store.ContentRepositoryStore;
import org.globalart.astrum.repository.maven.internal.Maven2Format;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(Maven2Format.NAME)
public class Maven2StoreProvider
    extends BespokeFormatStoreSupport<ContentRepositoryStore<Maven2ContentRepositoryDAO>, Maven2ComponentStore, // adds
                                                                                                                // support
                                                                                                                // for
                                                                                                                // the
                                                                                                                // base_version
                                                                                                                // column
        Maven2AssetStore, AssetBlobStore<Maven2AssetBlobDAO>, BrowseNodeStore<Maven2BrowseNodeDAO>>
{
  // nothing to add
}
