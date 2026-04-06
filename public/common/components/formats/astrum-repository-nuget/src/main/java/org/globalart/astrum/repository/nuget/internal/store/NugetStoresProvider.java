package org.globalart.astrum.repository.nuget.internal.store;

import org.globalart.astrum.repository.content.store.FormatStoreSupport;
import org.globalart.astrum.repository.nuget.internal.NugetFormat;
import org.globalart.astrum.repository.nuget.internal.browse.NugetBrowseNodeDAO;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(NugetFormat.NAME)
public class NugetStoresProvider
    extends FormatStoreSupport<NugetContentRepositoryDAO, NugetComponentDAO, NugetAssetDAO, NugetAssetBlobDAO, NugetBrowseNodeDAO>
{
}
