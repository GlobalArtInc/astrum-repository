package org.globalart.astrum.repository.docker.internal.store;

import org.globalart.astrum.repository.content.store.FormatStoreSupport;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.docker.internal.browse.DockerBrowseNodeDAO;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(DockerFormat.NAME)
public class DockerStoresProvider
    extends FormatStoreSupport<DockerContentRepositoryDAO, DockerComponentDAO, DockerAssetDAO, DockerAssetBlobDAO, DockerBrowseNodeDAO>
{
}
