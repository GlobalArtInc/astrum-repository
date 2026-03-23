
package org.globalart.astrum.repository.maven.internal;

import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.maven.VersionPolicy;

import static org.globalart.astrum.repository.maven.internal.Constants.METADATA_FILENAME;
import static org.globalart.astrum.repository.maven.internal.Constants.SNAPSHOT_VERSION_SUFFIX;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class VersionPolicyValidator
{
  private static final String METADATA_SNAPSHOT_PATH_SUFFIX = SNAPSHOT_VERSION_SUFFIX + "/" + METADATA_FILENAME;

  public boolean validArtifactPath(final VersionPolicy versionPolicy, final MavenPath.Coordinates coordinates) {
    if (versionPolicy == VersionPolicy.SNAPSHOT) {
      return coordinates.isSnapshot();
    }
    if (versionPolicy == VersionPolicy.RELEASE) {
      return !coordinates.isSnapshot();
    }
    return true;
  }

  public boolean validMetadataPath(final VersionPolicy versionPolicy, final String path) {
    boolean isMetadataSnapshot = path.endsWith(METADATA_SNAPSHOT_PATH_SUFFIX);
    if (isMetadataSnapshot && versionPolicy == VersionPolicy.RELEASE) {
      return false;
    }
    return true;
  }
}
