
package org.globalart.astrum.repository.content.store.example;

import org.globalart.astrum.repository.content.browse.store.BrowseNodeStore;
import org.globalart.astrum.repository.content.browse.store.example.TestBrowseNodeDAO;
import org.globalart.astrum.repository.content.store.AssetBlobStore;
import org.globalart.astrum.repository.content.store.BespokeFormatStoreSupport;
import org.globalart.astrum.repository.content.store.ComponentStore;
import org.globalart.astrum.repository.content.store.ContentRepositoryStore;

import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Bespoke store module for a bespoke format that uses a custom asset store with extra features.
 */
@Qualifier("test")
public class TestBespokeStoreProvider
    extends
    BespokeFormatStoreSupport<ContentRepositoryStore<TestContentRepositoryDAO>, ComponentStore<TestComponentDAO>, TestAssetStore, // adds
                                                                                                                                  // support
                                                                                                                                  // for
                                                                                                                                  // browseFlaggedAssets
        AssetBlobStore<TestAssetBlobDAO>, BrowseNodeStore<TestBrowseNodeDAO>>
{
  // nothing to add...
}
