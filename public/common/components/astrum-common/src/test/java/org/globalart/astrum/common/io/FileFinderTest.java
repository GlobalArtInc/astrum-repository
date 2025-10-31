
package org.globalart.astrum.common.io;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Optional;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class FileFinderTest
{
  @Rule
  public TemporaryFolder temporaryFolder = new TemporaryFolder();

  private static final byte[] PAYLOAD = "payload".getBytes(UTF_8);

  private File root;

  private final FileFinder fileFinder = new FileFinder();

  @Before
  public void prepare() throws IOException {
    root = temporaryFolder.newFolder("test-files");
    Path path = root.toPath();
    Files.write(path.resolve("file-2024-05-15-10-50-44.txt"), PAYLOAD);
    Files.write(path.resolve("file-2024-04-15-10-50-44.txt"), PAYLOAD);
    Files.write(path.resolve("file-2024-06-15-10-50-44.txt"), PAYLOAD);
  }

  @Test
  public void testFindLatestTimestampedFileWhenDirectoryEmpty() throws IOException {
    String prefix = "prefix";
    String suffix = "suffix";

    Optional<Path> result = fileFinder.findLatestTimestampedFile(root.toPath(), prefix, suffix);

    assertFalse(result.isPresent());
  }

  @Test
  public void testFindLatestTimestampedFileWhenDirectoryContainsMatchingFiles() throws IOException {
    String prefix = "file-";
    String suffix = ".txt";

    Optional<Path> result = fileFinder.findLatestTimestampedFile(root.toPath(), prefix, suffix);

    String expectedFileName = "file-2024-06-15-10-50-44.txt";
    assertTrue(result.isPresent());
    assertTrue(result.get().toString().contains(expectedFileName));
  }
}
