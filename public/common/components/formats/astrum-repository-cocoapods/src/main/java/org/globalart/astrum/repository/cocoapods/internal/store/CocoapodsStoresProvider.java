package org.globalart.astrum.repository.cocoapods.internal.store;

import org.globalart.astrum.repository.cocoapods.internal.CocoapodsFormat;
import org.globalart.astrum.repository.cocoapods.internal.browse.CocoapodsBrowseNodeDAO;
import org.globalart.astrum.repository.content.store.FormatStoreSupport;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(CocoapodsFormat.NAME)
public class CocoapodsStoresProvider
    extends FormatStoreSupport<CocoapodsContentRepositoryDAO, CocoapodsComponentDAO, CocoapodsAssetDAO, CocoapodsAssetBlobDAO, CocoapodsBrowseNodeDAO>
{
}
