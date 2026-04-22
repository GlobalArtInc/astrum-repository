package org.globalart.astrum.repository.composer.store;

import org.globalart.astrum.repository.composer.ComposerFormat;
import org.globalart.astrum.repository.composer.internal.browse.ComposerBrowseNodeDAO;
import org.globalart.astrum.repository.content.store.FormatStoreSupport;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(ComposerFormat.NAME)
public class ComposerStoresProvider
    extends FormatStoreSupport<ComposerContentRepositoryDAO, ComposerComponentDAO, ComposerAssetDAO, ComposerAssetBlobDAO, ComposerBrowseNodeDAO>
{
}
