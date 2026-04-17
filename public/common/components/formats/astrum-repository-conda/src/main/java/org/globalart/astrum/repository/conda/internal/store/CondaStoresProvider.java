package org.globalart.astrum.repository.conda.internal.store;

import org.globalart.astrum.repository.conda.CondaFormat;
import org.globalart.astrum.repository.conda.internal.browse.CondaBrowseNodeDAO;
import org.globalart.astrum.repository.content.store.FormatStoreSupport;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(CondaFormat.NAME)
public class CondaStoresProvider
    extends FormatStoreSupport<CondaContentRepositoryDAO, CondaComponentDAO, CondaAssetDAO, CondaAssetBlobDAO, CondaBrowseNodeDAO>
{
}
