
package org.globalart.astrum.capability;

import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.db.DatabaseCheck;

import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Primary
@Component
public class DefaultCapabilityDescriptorProvider
{
  private final Map<String, CapabilityDescriptor> descriptors;

  private final DatabaseCheck databaseCheck;

  @Autowired
  public DefaultCapabilityDescriptorProvider(
      final List<CapabilityDescriptor> descriptorsList,
      final DatabaseCheck databaseCheck)
  {
    this.descriptors = QualifierUtil.buildQualifierBeanMap(checkNotNull(descriptorsList));
    this.databaseCheck = checkNotNull(databaseCheck);
  }

  public List<CapabilityDescriptor> get() {
    return descriptors.values()
        .stream()
        .filter(capabilityDescriptor -> databaseCheck.isAllowedByVersion(capabilityDescriptor.getClass()))
        .collect(Collectors.toList());
  }
}
