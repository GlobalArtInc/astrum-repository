package org.globalart.astrum.content.npm.internal.store;

import org.globalart.astrum.content.npm.internal.browse.NpmBrowseNodeDAO;
import org.globalart.astrum.repository.content.store.FormatStoreSupport;
import org.globalart.astrum.repository.npm.internal.NpmFormat;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(NpmFormat.NAME)
public class NpmStoresProvider
    extends FormatStoreSupport<NpmContentRepositoryDAO, NpmComponentDAO, NpmAssetDAO, NpmAssetBlobDAO, NpmBrowseNodeDAO>
{
}
