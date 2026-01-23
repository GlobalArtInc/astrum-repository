
package org.globalart.astrum.blobstore;

import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.stream.Collectors;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.db.DatabaseCheck;
import org.springframework.stereotype.Component;

@Component
public class BlobStoreDescriptorProvider
{
  private final Map<String, BlobStoreDescriptor> blobStoreDescriptors;

  private final DatabaseCheck databaseCheck;

  @Autowired
  public BlobStoreDescriptorProvider(
      final DatabaseCheck databaseCheck,
      final List<BlobStoreDescriptor> blobStoreDescriptorsList)
  {
    this.databaseCheck = databaseCheck;
    this.blobStoreDescriptors = QualifierUtil.buildQualifierBeanMap(blobStoreDescriptorsList);
  }

  public Map<String, BlobStoreDescriptor> get() {
    return blobStoreDescriptors.entrySet()
        .stream()
        .filter(item -> databaseCheck.isAllowedByVersion(item.getValue().getClass()))
        .collect(Collectors.toMap(Entry::getKey, Entry::getValue));
  }
}
