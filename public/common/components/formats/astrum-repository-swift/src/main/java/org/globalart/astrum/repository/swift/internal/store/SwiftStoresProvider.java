package org.globalart.astrum.repository.swift.internal.store;

import org.globalart.astrum.repository.content.browse.store.BrowseNodeStore;
import org.globalart.astrum.repository.content.store.AssetBlobStore;
import org.globalart.astrum.repository.content.store.AssetStore;
import org.globalart.astrum.repository.content.store.BespokeFormatStoreSupport;
import org.globalart.astrum.repository.content.store.ContentRepositoryStore;
import org.globalart.astrum.repository.swift.SwiftFormat;
import org.globalart.astrum.repository.swift.internal.browse.SwiftBrowseNodeDAO;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(SwiftFormat.NAME)
public class SwiftStoresProvider
    extends
    BespokeFormatStoreSupport<ContentRepositoryStore<SwiftContentRepositoryDAO>, SwiftComponentStore, AssetStore<SwiftAssetDAO>, AssetBlobStore<SwiftAssetBlobDAO>, BrowseNodeStore<SwiftBrowseNodeDAO>>
{
}
