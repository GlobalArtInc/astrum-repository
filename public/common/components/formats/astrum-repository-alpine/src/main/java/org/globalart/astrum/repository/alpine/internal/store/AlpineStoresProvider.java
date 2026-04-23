package org.globalart.astrum.repository.alpine.internal.store;

import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.alpine.internal.browse.AlpineBrowseNodeDAO;
import org.globalart.astrum.repository.content.store.FormatStoreSupport;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(AlpineFormat.NAME)
public class AlpineStoresProvider
    extends FormatStoreSupport<AlpineContentRepositoryDAO, AlpineComponentDAO, AlpineAssetDAO, AlpineAssetBlobDAO, AlpineBrowseNodeDAO>
{
}
