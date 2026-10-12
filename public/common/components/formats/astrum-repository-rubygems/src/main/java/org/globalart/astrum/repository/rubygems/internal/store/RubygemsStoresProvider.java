package org.globalart.astrum.repository.rubygems.internal.store;

import org.globalart.astrum.repository.content.store.FormatStoreSupport;
import org.globalart.astrum.repository.rubygems.RubygemsFormat;
import org.globalart.astrum.repository.rubygems.internal.browse.RubygemsBrowseNodeDAO;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(RubygemsFormat.NAME)
public class RubygemsStoresProvider
    extends FormatStoreSupport<RubygemsContentRepositoryDAO, RubygemsComponentDAO, RubygemsAssetDAO, RubygemsAssetBlobDAO, RubygemsBrowseNodeDAO>
{
}
