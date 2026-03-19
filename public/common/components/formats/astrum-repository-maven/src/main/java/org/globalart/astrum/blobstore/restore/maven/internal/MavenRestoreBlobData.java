
package org.globalart.astrum.blobstore.restore.maven.internal;

import java.util.Properties;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.restore.datastore.DataStoreRestoreBlobData;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.maven.MavenPathParser;

import static com.google.common.base.Preconditions.checkNotNull;

public class MavenRestoreBlobData
    extends DataStoreRestoreBlobData
{
  private final MavenPath mavenPath;

  public MavenRestoreBlobData(
      final Blob blob,
      final Properties blobProperties,
      final BlobStore blobStore,
      final RepositoryManager repositoryManager,
      final MavenPathParser mavenPathParser)
  {
    super(blob, blobProperties, blobStore, repositoryManager);
    this.mavenPath = checkNotNull(mavenPathParser).parsePath(getBlobName());
  }

  public MavenPath getMavenPath() {
    return mavenPath;
  }
}
