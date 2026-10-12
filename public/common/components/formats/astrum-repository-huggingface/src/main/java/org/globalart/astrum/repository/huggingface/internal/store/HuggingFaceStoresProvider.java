package org.globalart.astrum.repository.huggingface.internal.store;

import org.globalart.astrum.repository.content.store.FormatStoreSupport;
import org.globalart.astrum.repository.huggingface.HuggingFaceFormat;
import org.globalart.astrum.repository.huggingface.internal.browse.HuggingFaceBrowseNodeDAO;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(HuggingFaceFormat.NAME)
public class HuggingFaceStoresProvider
    extends FormatStoreSupport<HuggingFaceContentRepositoryDAO, HuggingFaceComponentDAO, HuggingFaceAssetDAO, HuggingFaceAssetBlobDAO, HuggingFaceBrowseNodeDAO>
{
}
