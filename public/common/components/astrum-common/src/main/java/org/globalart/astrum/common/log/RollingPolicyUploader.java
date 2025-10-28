
package org.globalart.astrum.common.log;

/**
 * Interface for uploading log files to a remote location.
 */
public interface RollingPolicyUploader
{
  /**
   * Rollover the given path with the given parameters.
   *
   * @param context the context prefix to use in the remote location for the given paths
   * @param datePath the log date path to use in the remote location for the given paths
   * @param path the path to the file to upload
   */
  void rollover(final String context, final String datePath, final String path);
}
