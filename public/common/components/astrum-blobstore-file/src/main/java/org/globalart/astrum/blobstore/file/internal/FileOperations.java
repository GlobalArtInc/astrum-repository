
package org.globalart.astrum.blobstore.file.internal;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Path;
import java.time.OffsetDateTime;

import org.globalart.astrum.blobstore.StreamMetrics;

/**
 * A wrapper around file operations to make mocking easier.
 *
 * @since 3.0
 */
public interface FileOperations
{
  /**
   * Creates a file (and its containing directories, if necessary) and populates it from the
   * InputStream, which gets closed.
   *
   * @return Basic metrics about the stream.
   */
  StreamMetrics create(Path path, InputStream data) throws IOException;

  /**
   * Creates a hard link.
   */
  void hardLink(Path source, Path newLink) throws IOException;

  /**
   * Copies a file.
   *
   * @since 3.2
   */
  void copy(Path source, Path target) throws IOException;

  /**
   * Moves a file without any guarantee of atomicity.
   *
   * @since 3.2
   */
  void move(Path source, Path target) throws IOException;

  /**
   * Moves a file atomically, throwing <code>AtomicMoveNotSupportedException</code> if not supported.
   *
   * @since 3.2
   */
  void moveAtomic(Path source, Path target) throws IOException;

  /**
   * Overwrites a file without any guarantee of atomicity.
   *
   * @since 3.8
   */
  void overwrite(Path source, Path target) throws IOException;

  /**
   * Overwrites a file atomically, throwing <code>AtomicMoveNotSupportedException</code> if not supported.
   *
   * @since 3.8
   */
  void overwriteAtomic(Path source, Path target) throws IOException;

  /**
   * Moves a file, falling back on copy/delete if a <code>FileSystemException</code> is thrown.
   *
   * @since 3.5
   */
  void copyIfLocked(final Path source, final Path target, Mover mover) throws IOException;

  /**
   * Computes basic metrics about the file.
   */
  StreamMetrics computeMetrics(Path file) throws IOException;

  boolean exists(Path path);

  boolean isBlobZeroLength(Path path);

  InputStream openInputStream(Path path) throws IOException;

  /**
   * Returns true if the file existed before deletion, false otherwise.
   */
  boolean delete(Path path) throws IOException;

  /**
   * Deletes the path and does not throw exceptions on failure
   *
   * @param path to be deleted
   * @return {@code true} if the path was deleted, {@code false} otherwise
   *
   * @since 3.15
   */
  boolean deleteQuietly(Path path);

  /**
   * Recursively deletes all files and subdirectories, then the directory itself.
   */
  void deleteDirectory(Path directory) throws IOException;

  /**
   * Returns true if the directory was empty and could be removed, false otherwise.
   */
  boolean deleteEmptyDirectory(Path directory);

  OffsetDateTime lastModified(Path path) throws IOException;

  /**
   * @since 3.5
   */
  @FunctionalInterface
  interface Mover
  {
    void accept(Path source, Path destination) throws IOException;
  }
}
