
package org.globalart.astrum.internal.provisioning;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.BlobStoreApi;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreManager;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * @since 3.0
 */
@Component
public class BlobStoreApiImpl
    implements BlobStoreApi
{
  private final BlobStoreManager blobStoreManager;

  @Autowired
  public BlobStoreApiImpl(final BlobStoreManager blobStoreManager) {
    this.blobStoreManager = checkNotNull(blobStoreManager);
  }

  @Override
  public BlobStoreConfiguration createFileBlobStore(final String name, final String path) {
    Map<String, Map<String, Object>> attributes = new HashMap<>();
    Map<String, Object> fileAttributes = new HashMap<>();
    fileAttributes.put("path", checkNotNull(path));
    attributes.put("file", fileAttributes);
    BlobStoreConfiguration blobStoreConfiguration = blobStoreManager.newConfiguration();
    blobStoreConfiguration.setName(name);
    blobStoreConfiguration.setType("File");
    blobStoreConfiguration.setAttributes(attributes);
    return doCreate(blobStoreConfiguration);
  }

  @Override
  public BlobStoreConfiguration createBlobStoreGroup(
      final String name,
      final List<String> memberNames,
      final String fillPolicy)
  {
    Map<String, Map<String, Object>> attributes = new HashMap<>();
    Map<String, Object> groupAttributes = new HashMap<>();
    groupAttributes.put("members", memberNames);
    groupAttributes.put("fillPolicy", fillPolicy);
    attributes.put("group", groupAttributes);
    BlobStoreConfiguration blobStoreConfiguration = blobStoreManager.newConfiguration();
    blobStoreConfiguration.setName(name);
    blobStoreConfiguration.setType("Group");
    blobStoreConfiguration.setAttributes(attributes);
    return doCreate(blobStoreConfiguration);
  }

  @Override
  public BlobStoreConfiguration createS3BlobStore(final String name, final Map<String, String> s3Config) {
    // re-collecting to move from <String,String> to <String,Object>
    Map<String, Map<String, Object>> attributes = new HashMap<>();
    attributes.put("s3", new HashMap<>(s3Config));
    BlobStoreConfiguration blobStoreConfiguration = blobStoreManager.newConfiguration();
    blobStoreConfiguration.setName(name);
    blobStoreConfiguration.setType("S3");
    blobStoreConfiguration.setAttributes(attributes);
    return doCreate(blobStoreConfiguration);
  }

  private BlobStoreConfiguration doCreate(final BlobStoreConfiguration blobStoreConfiguration) {
    try {
      return blobStoreManager.create(blobStoreConfiguration).getBlobStoreConfiguration();
    }
    catch (Exception e) {
      throw new RuntimeException(e);
    }
  }
}
