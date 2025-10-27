
package org.globalart.astrum.common.io;

import java.io.IOException;
import java.nio.file.DirectoryStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Comparator;
import java.util.Optional;
import java.util.Set;
import java.util.function.Predicate;
import java.util.stream.Stream;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class FileFinder
{
  private static final Logger log = LoggerFactory.getLogger(FileFinder.class);

  public static Optional<Path> findLatestTimestampedFile(
      Path directoryPath,
      String prefix,
      String suffix) throws IOException
  {
    try (Stream<Path> stream = Files.list(directoryPath)) {
      Predicate<Path> nameCheck = path -> matchesPattern(path, prefix, suffix);
      Comparator<Path> timestampComparator = Comparator.comparingLong(path -> getFileTimestamp(path, prefix, suffix));
      return stream.filter(nameCheck).max(timestampComparator);
    }
  }

  public static boolean pathContainsFolder(Path path, Set<String> folderNames) {
    if (!Files.isDirectory(path)) {
      return false;
    }

    try (DirectoryStream<Path> directoryStream = Files.newDirectoryStream(path)) {
      for (Path entry : directoryStream) {
        if (Files.isDirectory(entry) && folderNames.contains(entry.getFileName().toString())) {
          return true;
        }
      }
    }
    catch (IOException e) {
      log.error("Error reading directory stream for path: " + path.toString(), e);
    }

    return false;
  }

  private static boolean matchesPattern(Path path, String prefix, String suffix) {
    String fileName = path.getFileName().toString();
    return fileName.startsWith(prefix) && fileName.endsWith(suffix);
  }

  private static long getFileTimestamp(Path path, String prefix, String suffix) {
    String fileName = path.getFileName().toString();
    String timestampStr = fileName.substring(prefix.length(), fileName.length() - suffix.length());
    return Long.parseLong(timestampStr.replace("-", ""));
  }
}
