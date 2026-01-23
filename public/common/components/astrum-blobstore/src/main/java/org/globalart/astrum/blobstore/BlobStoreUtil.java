
package org.globalart.astrum.blobstore;

import java.util.Map;

import org.globalart.astrum.common.hash.HashAlgorithm;

import com.google.common.hash.HashCode;

/**
 * Blob store utilities.
 *
 * @since 3.15
 */
public interface BlobStoreUtil
{
  /**
   * Get a count of the repositories using a blob store.
   *
   * @param blobStoreId blob store id
   * @return number of repositories using the blob store
   */
  int usageCount(String blobStoreId);

  /**
   * Returns true if the file path is valid. The path is valid if all the folder names in the path are less than the
   * given maximum.
   * 
   * @since 3.20
   * @param filePath A file path
   * @param maxLength The max length
   * @return True if valid, false otherwise
   */
  boolean validateFilePath(String filePath, int maxLength);

  Map<HashAlgorithm, HashCode> toHashObjects(final Map<String, String> checksums);
}
