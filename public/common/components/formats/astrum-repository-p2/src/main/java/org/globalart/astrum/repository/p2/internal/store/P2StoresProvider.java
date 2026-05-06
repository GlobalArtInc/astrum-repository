package org.globalart.astrum.repository.p2.internal.store;

import org.globalart.astrum.repository.content.store.FormatStoreSupport;
import org.globalart.astrum.repository.p2.internal.P2Format;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(P2Format.NAME)
public class P2StoresProvider
    extends FormatStoreSupport<P2ContentRepositoryDAO, P2ComponentDAO, P2AssetDAO, P2AssetBlobDAO, P2BrowseNodeDAO>
{
}
