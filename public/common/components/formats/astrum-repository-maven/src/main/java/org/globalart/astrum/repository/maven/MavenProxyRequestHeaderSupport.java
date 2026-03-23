

package org.globalart.astrum.repository.maven;

import java.util.Collection;
import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityReferenceFilterBuilder;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.utils.httpclient.UserAgentGenerator;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

@Component
public class MavenProxyRequestHeaderSupport
{
  private static final String ANALYTICS_CAPABILITY = "analytics-configuration";

  private final CapabilityRegistry capabilityRegistry;

  private final UserAgentGenerator userAgentGenerator;

  @Autowired
  public MavenProxyRequestHeaderSupport(
      final CapabilityRegistry capabilityRegistry,
      final UserAgentGenerator userAgentGenerator)
  {
    this.capabilityRegistry = checkNotNull(capabilityRegistry);
    this.userAgentGenerator = checkNotNull(userAgentGenerator);
  }

  public String getUserAgentForAnalytics() {
    CapabilityReference capabilityReference = getCapabilityReference();
    return userAgentGenerator.buildUserAgentForAnalytics(capabilityReference);
  }

  @Nullable
  public CapabilityReference getCapabilityReference() {
    CapabilityType capabilityType = CapabilityType.capabilityType(ANALYTICS_CAPABILITY);
    Collection<? extends CapabilityReference> refs = capabilityRegistry.get(
        CapabilityReferenceFilterBuilder.capabilities().withType(capabilityType).includeNotExposed());
    CapabilityReference capabilityReference = null;
    if (!refs.isEmpty()) {
      capabilityReference = refs.iterator().next();
    }
    return capabilityReference;
  }

}
