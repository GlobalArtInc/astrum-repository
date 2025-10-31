
package org.globalart.astrum.common.wonderland;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;

import javax.annotation.Nullable;

/**
 * Download service.
 *
 * @since 2.8
 */
public interface DownloadService
{

  /**
   * @since 3.13
   */
  class Download
  {

    private final long length;

    private final InputStream bytes;

    public Download(final long length, final InputStream bytes) {
      this.length = length;
      this.bytes = bytes;
    }

    /**
     * @since 3.16
     */
    public long getLength() {
      return length;
    }

    /**
     * @since 3.16
     */
    public InputStream getBytes() {
      return bytes;
    }
  }

  /**
   * @param fileName of file to be downloaded
   * @param authTicket authentication ticket
   * @return the download, or null if it doesn't exist
   */
  @Nullable
  Download get(String fileName, String authTicket) throws IOException;

  /**
   * Moves specified file to downloads, using specified name.
   *
   * @param source to be moved
   * @param name name of file in downloads dir
   * @return moved filename (residing in downloads)
   */
  String move(File source, String name) throws IOException;

  /**
   * Generate a unique file prefix.
   */
  String uniqueName(String prefix);
}
