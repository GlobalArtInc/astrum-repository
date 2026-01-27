
package org.globalart.astrum.blobstore.s3.internal.capability;

import java.util.Map;

import org.globalart.astrum.capability.CapabilitySupport;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component(CustomS3RegionCapabilityDescriptor.TYPE_ID)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CustomS3RegionCapability
    extends CapabilitySupport<CustomS3RegionCapabilityConfiguration>
{
  @Override
  protected CustomS3RegionCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new CustomS3RegionCapabilityConfiguration(properties);
  }
}
