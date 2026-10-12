package org.globalart.astrum.repository.yum.internal.store;

import org.globalart.astrum.repository.content.store.FormatStoreSupport;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.repository.yum.internal.browse.YumBrowseNodeDAO;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(YumFormat.NAME)
public class YumStoresProvider
    extends FormatStoreSupport<YumContentRepositoryDAO, YumComponentDAO, YumAssetDAO, YumAssetBlobDAO, YumBrowseNodeDAO>
{
}
