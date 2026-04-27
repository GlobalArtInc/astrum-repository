package org.globalart.astrum.repository.r.internal.store;

import org.globalart.astrum.repository.content.store.FormatStoreSupport;
import org.globalart.astrum.repository.r.RFormat;
import org.globalart.astrum.repository.r.internal.browse.RBrowseNodeDAO;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(RFormat.NAME)
public class RStoresProvider
    extends FormatStoreSupport<RContentRepositoryDAO, RComponentDAO, RAssetDAO, RAssetBlobDAO, RBrowseNodeDAO>
{
}
