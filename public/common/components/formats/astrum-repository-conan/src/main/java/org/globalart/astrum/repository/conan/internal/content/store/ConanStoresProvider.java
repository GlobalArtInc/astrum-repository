package org.globalart.astrum.repository.conan.internal.content.store;

import org.globalart.astrum.repository.conan.ConanFormat;
import org.globalart.astrum.repository.conan.internal.content.browse.ConanBrowseNodeDAO;
import org.globalart.astrum.repository.content.store.FormatStoreSupport;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(ConanFormat.NAME)
public class ConanStoresProvider
    extends FormatStoreSupport<ConanContentRepositoryDAO, ConanComponentDAO, ConanAssetDAO, ConanAssetBlobDAO,
    ConanBrowseNodeDAO>
{
}
