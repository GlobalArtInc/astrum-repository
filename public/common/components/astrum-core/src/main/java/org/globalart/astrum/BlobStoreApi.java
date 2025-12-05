
package org.globalart.astrum;

import java.util.List;
import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.common.script.ScriptApi;

/**
 * BlobStore provisioning capabilities of the repository manager.
 * 
 * @since 3.0
 */
public interface BlobStoreApi
    extends ScriptApi
{
  default String getName() {
    return "blobStore";
  }

  /**
   * Create a new File based BlobStore.
   * 
   * @param name the name for the new BlobStore
   * @param path the path where the BlobStore should store data
   */
  BlobStoreConfiguration createFileBlobStore(String name, String path);

  /**
   * Create a new BlobStore group.
   *
   * @param name the name for the new BlobStore
   * @param memberNames name of the member BlobStores
   * @param fillPolicy name of the fill policy
   * @since 3.14
   */
  BlobStoreConfiguration createBlobStoreGroup(String name, List<String> memberNames, String fillPolicy);

  /**
   * Create a new S3 based BlobStore.
   *
   * @param name the name for the new BlobStore
   * @param config the configuration map for the new blobstore
   * @since 3.6
   */
  BlobStoreConfiguration createS3BlobStore(String name, Map<String, String> config);
}
