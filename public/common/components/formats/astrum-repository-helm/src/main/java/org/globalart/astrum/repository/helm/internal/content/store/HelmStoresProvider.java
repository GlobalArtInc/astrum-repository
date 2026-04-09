package org.globalart.astrum.repository.helm.internal.content.store;

import org.globalart.astrum.repository.content.store.FormatStoreSupport;
import org.globalart.astrum.repository.helm.internal.HelmFormat;
import org.globalart.astrum.repository.helm.internal.content.browse.HelmBrowseNodeDAO;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(HelmFormat.NAME)
public class HelmStoresProvider
    extends FormatStoreSupport<HelmContentRepositoryDAO, HelmComponentDAO, HelmAssetDAO, HelmAssetBlobDAO, HelmBrowseNodeDAO>
{
}
