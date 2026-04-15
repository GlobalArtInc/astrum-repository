package org.globalart.astrum.repository.golang.internal.content.store;

import org.globalart.astrum.repository.content.store.FormatStoreSupport;
import org.globalart.astrum.repository.golang.GolangFormat;
import org.globalart.astrum.repository.golang.internal.content.browse.GoBrowseNodeDAO;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(GolangFormat.NAME)
public class GoStoresProvider
    extends FormatStoreSupport<GoContentRepositoryDAO, GoComponentDAO, GoAssetDAO, GoAssetBlobDAO, GoBrowseNodeDAO>
{
}
