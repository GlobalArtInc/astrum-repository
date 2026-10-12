package org.globalart.astrum.repository.gitlfs.internal.browse;

import org.globalart.astrum.repository.content.browse.AssetPathBrowseNodeGenerator;
import org.globalart.astrum.repository.gitlfs.GitLfsFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * Lays objects out by their asset path, {@code objects/<first two>/<next two>/<oid>}, so browse groups them by the
 * prefix of their id.
 */
@Component
@Qualifier(GitLfsFormat.NAME)
public class GitLfsBrowseNodeGenerator
    extends AssetPathBrowseNodeGenerator
{
}
