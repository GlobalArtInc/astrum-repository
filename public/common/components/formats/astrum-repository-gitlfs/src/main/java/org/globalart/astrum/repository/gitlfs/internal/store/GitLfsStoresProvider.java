package org.globalart.astrum.repository.gitlfs.internal.store;

import org.globalart.astrum.repository.content.store.FormatStoreSupport;
import org.globalart.astrum.repository.gitlfs.GitLfsFormat;
import org.globalart.astrum.repository.gitlfs.internal.browse.GitLfsBrowseNodeDAO;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(GitLfsFormat.NAME)
public class GitLfsStoresProvider
    extends FormatStoreSupport<GitLfsContentRepositoryDAO, GitLfsComponentDAO, GitLfsAssetDAO, GitLfsAssetBlobDAO, GitLfsBrowseNodeDAO>
{
}
