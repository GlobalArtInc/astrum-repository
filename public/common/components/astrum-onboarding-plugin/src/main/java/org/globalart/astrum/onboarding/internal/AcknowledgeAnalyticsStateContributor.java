
package org.globalart.astrum.onboarding.internal;

import java.util.HashMap;
import java.util.Map;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.capability.CapabilityReferenceFilterBuilder.CapabilityReferenceFilter;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.rapture.StateContributor;

import static org.globalart.astrum.capability.CapabilityType.capabilityType;
import org.springframework.stereotype.Component;

/**
 * State contributor defining whether acknowledgement of the analytics submission state is required.
 *
 * @since 3.35
 */
@Component
public class AcknowledgeAnalyticsStateContributor
    implements StateContributor
{
  protected static final String OSS = "OSS";

  private static final String ANALYTICS_CONFIGURATION = "analytics-configuration";

  private final ApplicationVersion applicationVersion;

  private final CapabilityRegistry capabilityRegistry;

  @Autowired
  public AcknowledgeAnalyticsStateContributor(
      final ApplicationVersion applicationVersion,
      final CapabilityRegistry capabilityRegistry)
  {

    this.applicationVersion = applicationVersion;
    this.capabilityRegistry = capabilityRegistry;
  }

  @Nullable
  @Override
  public Map<String, Object> getState() {
    Map<String, Object> properties = new HashMap<>();

    properties.put("acknowledgeAnalytics.required", applies());

    return properties;
  }

  private boolean applies() {
    return OSS.equals(applicationVersion.getEdition()) && analyticsCapabilityAbsent();
  }

  private boolean analyticsCapabilityAbsent() {
    CapabilityType capabilityType = capabilityType(ANALYTICS_CONFIGURATION);
    return capabilityRegistry.get(new CapabilityReferenceFilter().withType(capabilityType).includeNotExposed())
        .isEmpty();
  }
}
