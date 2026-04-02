package org.globalart.astrum.repository.pypi.internal.content.store;

import org.globalart.astrum.repository.content.store.FormatStoreSupport;
import org.globalart.astrum.repository.pypi.internal.PyPiFormat;
import org.globalart.astrum.repository.pypi.internal.content.browse.PypiBrowseNodeDAO;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(PyPiFormat.NAME)
public class PypiStoresProvider
    extends FormatStoreSupport<PypiContentRepositoryDAO, PypiComponentDAO, PypiAssetDAO, PypiAssetBlobDAO, PypiBrowseNodeDAO>
{
}
