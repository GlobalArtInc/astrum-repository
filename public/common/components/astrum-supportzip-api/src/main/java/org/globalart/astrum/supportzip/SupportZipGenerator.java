
package org.globalart.astrum.supportzip;

import java.io.OutputStream;
import java.io.Serializable;

import org.globalart.astrum.common.log.SupportZipGeneratorRequest;

/**
 * Generates a support ZIP file.
 *
 * @since 2.7
 */
public interface SupportZipGenerator
{
  /**
   * Result of support ZIP generate request.
   */
  class Result
      implements Serializable
  {
    private static final long serialVersionUID = -3253827134366752809L;

    private boolean truncated;

    private String filename;

    private String localPath;

    private long size;

    public Result(final boolean truncated, final String filename, final String localPath, final long size) {
      this.truncated = truncated;
      this.filename = filename;
      this.localPath = localPath;
      this.size = size;
    }

    /**
     * True if the ZIP or any of its contents had been truncated.
     */
    public boolean isTruncated() {
      return truncated;
    }

    /**
     * The name of the generated ZIP file.
     */
    public String getFilename() {
      return filename;
    }

    /**
     * The local path of the generated ZIP file.
     */
    public String getLocalPath() {
      return localPath;
    }

    /**
     * The size of the generated ZIP file.
     */
    public long getSize() {
      return size;
    }

    @Override
    public String toString() {
      return getClass().getSimpleName() + "{" +
          "truncated=" + truncated +
          ", filename=" + filename +
          ", localPath=" + localPath +
          ", size=" + size +
          '}';
    }
  }

  /**
   * Generate a support ZIP for the given request.
   */
  Result generate(SupportZipGeneratorRequest request);

  /**
   * Generate a support ZIP for the given request with a custom prefix.
   */
  Result generate(SupportZipGeneratorRequest request, String prefix);

  /**
   * Generate a support ZIP.
   * 
   * @param outputStream the output stream to write the ZIP to.
   * @param prefix directory prefix applied to files in the ZIP.
   * @return true if the zip was truncated.
   * @since 3.5
   */
  boolean generate(SupportZipGeneratorRequest request, String prefix, OutputStream outputStream);
}
