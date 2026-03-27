
package org.globalart.astrum.repository.maven.internal.hosted.metadata;

import java.io.IOException;
import java.util.Optional;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.maven.MavenPath;

import org.apache.maven.artifact.repository.metadata.Metadata;
import org.apache.maven.artifact.repository.metadata.Versioning;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.doReturn;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.verify;

/**
 * Tests for {@link AbstractMetadataUpdater} focusing on the writeIfChanged behavior with missing checksums.
 */
@RunWith(MockitoJUnitRunner.class)
public class AbstractMetadataUpdaterTest
{
  @Mock
  private Repository repository;

  @Mock
  private MavenPath mavenPath;

  private TestMetadataUpdater metadataUpdater;

  @Before
  public void setup() {
    metadataUpdater = spy(new TestMetadataUpdater(true, repository, true));
  }

  @Test
  public void writeIsCalledWhenMetadataContentChanges() throws Exception {
    // Create metadata objects with different content
    Metadata oldMetadata = createMetadata("org.test", "artifact", "1.0.0");
    Metadata newMetadata = createMetadata("org.test", "artifact", "2.0.0");

    doReturn(false).when(metadataUpdater).hasMissingChecksums(mavenPath);

    metadataUpdater.writeIfChanged(mavenPath, oldMetadata, newMetadata);

    verify(metadataUpdater).write(mavenPath, newMetadata);
  }

  @Test
  public void writeIsCalledWhenChecksumsAreMissingEvenIfContentUnchanged() throws Exception {
    // Create identical metadata objects
    Metadata oldMetadata = createMetadata("org.test", "artifact", "1.0.0");
    Metadata newMetadata = createMetadata("org.test", "artifact", "1.0.0");

    doReturn(true).when(metadataUpdater).hasMissingChecksums(mavenPath);

    metadataUpdater.writeIfChanged(mavenPath, oldMetadata, newMetadata);

    verify(metadataUpdater).write(mavenPath, newMetadata);
  }

  @Test
  public void writeIsNotCalledWhenContentUnchangedAndAllChecksumsExist() throws Exception {
    // Create identical metadata objects
    Metadata oldMetadata = createMetadata("org.test", "artifact", "1.0.0");
    Metadata newMetadata = createMetadata("org.test", "artifact", "1.0.0");

    doReturn(false).when(metadataUpdater).hasMissingChecksums(mavenPath);

    metadataUpdater.writeIfChanged(mavenPath, oldMetadata, newMetadata);

    verify(metadataUpdater, never()).write(any(MavenPath.class), any(Metadata.class));
  }

  @Test
  public void writeIsCalledWhenContentChangesAndChecksumsMissing() throws Exception {
    // Create metadata objects with different content
    Metadata oldMetadata = createMetadata("org.test", "artifact", "1.0.0");
    Metadata newMetadata = createMetadata("org.test", "artifact", "2.0.0");

    doReturn(true).when(metadataUpdater).hasMissingChecksums(mavenPath);

    metadataUpdater.writeIfChanged(mavenPath, oldMetadata, newMetadata);

    verify(metadataUpdater).write(mavenPath, newMetadata);
  }

  @Test
  public void hasMissingChecksumsIsNotCalledWhenRebuildChecksumsFlagIsFalse() throws Exception {
    // Create updater with rebuildChecksums=false (normal upload/delete path)
    metadataUpdater = spy(new TestMetadataUpdater(true, repository, false));

    // Create identical metadata objects
    Metadata oldMetadata = createMetadata("org.test", "artifact", "1.0.0");
    Metadata newMetadata = createMetadata("org.test", "artifact", "1.0.0");

    metadataUpdater.writeIfChanged(mavenPath, oldMetadata, newMetadata);

    // Verify hasMissingChecksums is never called when flag is false
    verify(metadataUpdater, never()).hasMissingChecksums(any(MavenPath.class));
    // Verify write is not called since content unchanged and checksums not checked
    verify(metadataUpdater, never()).write(any(MavenPath.class), any(Metadata.class));
  }

  /**
   * Creates a Metadata object with the specified groupId, artifactId, and version.
   */
  private Metadata createMetadata(final String groupId, final String artifactId, final String version) {
    Metadata metadata = new Metadata();
    metadata.setGroupId(groupId);
    metadata.setArtifactId(artifactId);
    Versioning versioning = new Versioning();
    versioning.setLatest(version);
    versioning.setRelease(version);
    versioning.addVersion(version);
    metadata.setVersioning(versioning);
    return metadata;
  }

  /**
   * Testable subclass that allows mocking the abstract hasMissingChecksums method.
   */
  private class TestMetadataUpdater
      extends AbstractMetadataUpdater
  {
    public TestMetadataUpdater(final boolean update, final Repository repository) {
      super(update, repository);
    }

    public TestMetadataUpdater(
        final boolean update,
        final Repository repository,
        final boolean rebuildChecksums)
    {
      super(update, repository, rebuildChecksums);
    }

    @Override
    protected boolean hasMissingChecksums(final MavenPath mavenPath) {
      return false;
    }

    @Override
    protected void write(final MavenPath mavenPath, final Metadata metadata) throws IOException {
      // Mock implementation - tracked by spy
    }

    @Override
    protected Optional<Metadata> read(final MavenPath mavenPath) throws IOException {
      return Optional.empty();
    }

    @Override
    protected void delete(final MavenPath mavenPath) {
      // Mock implementation
    }
  }
}
