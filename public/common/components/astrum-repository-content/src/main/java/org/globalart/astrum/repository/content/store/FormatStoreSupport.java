
package org.globalart.astrum.repository.content.store;

import org.globalart.astrum.repository.content.browse.store.BrowseNodeDAO;
import org.globalart.astrum.repository.content.browse.store.BrowseNodeStore;

public abstract class FormatStoreSupport<CONTENT_REPOSITORY_DAO extends ContentRepositoryDAO, COMPONENT_DAO extends ComponentDAO, ASSET_DAO extends AssetDAO, ASSET_BLOB_DAO extends AssetBlobDAO, BROWSE_NODE_DAO extends BrowseNodeDAO>
    extends
    BespokeFormatStoreSupport<ContentRepositoryStore<CONTENT_REPOSITORY_DAO>, ComponentStore<COMPONENT_DAO>, AssetStore<ASSET_DAO>, AssetBlobStore<ASSET_BLOB_DAO>, BrowseNodeStore<BROWSE_NODE_DAO>>
{

}
