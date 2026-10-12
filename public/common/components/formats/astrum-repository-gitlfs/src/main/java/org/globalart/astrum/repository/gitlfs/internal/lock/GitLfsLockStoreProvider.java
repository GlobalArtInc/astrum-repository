package org.globalart.astrum.repository.gitlfs.internal.lock;

import org.globalart.astrum.repository.content.store.ContentStoreBeanDefinitionRegistryPostProcessorSupport;
import org.globalart.astrum.repository.gitlfs.GitLfsFormat;

import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Binds the lock store of the Git LFS format in each content data store.
 */
@Qualifier(GitLfsFormat.NAME)
public class GitLfsLockStoreProvider
    extends ContentStoreBeanDefinitionRegistryPostProcessorSupport<GitLfsLockStore>
{
}
