
package org.globalart.astrum.repository.content.tasks;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

@AvailabilityVersion(from = "1.0")
@Component
public class CreateAssetBlobIndexTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "repository.asset.blob.index.migration";

  @Autowired
  public CreateAssetBlobIndexTaskDescriptor(
      @Value("${repository.asset.blob.index.migration.exposed:false}") final boolean exposed)
  {
    super(TYPE_ID, CreateAssetBlobIndexTask.class, "Create asset blob index for blob_created and asset_blob_id",
        true, exposed, true);
  }
}
