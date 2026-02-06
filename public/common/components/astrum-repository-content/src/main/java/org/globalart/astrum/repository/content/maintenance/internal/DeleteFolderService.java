
package org.globalart.astrum.repository.content.maintenance.internal;

import java.time.OffsetDateTime;

import org.globalart.astrum.repository.Repository;

/**
 * Deletes assets under a given path, created before a given timestamp, in a repository based on a user's privileges.
 *
 * @since 3.26
 */
public interface DeleteFolderService
{
  void deleteFolder(Repository repository, String path, OffsetDateTime timestamp);
}
