
package org.globalart.astrum.repository.httpbridge.legacy;

import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.capability.CapabilityDescriptorSupport;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.capability.Tag;
import org.globalart.astrum.capability.Taggable;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.FormField;

import static org.globalart.astrum.capability.Tag.categoryTag;
import static org.globalart.astrum.capability.Tag.tags;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * {@link LegacyUrlCapability} descriptor.
 *
 * @since 3.7
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(LegacyUrlCapabilityDescriptor.TYPE_ID)
public class LegacyUrlCapabilityDescriptor
    extends CapabilityDescriptorSupport<LegacyUrlCapabilityConfiguration>
    implements Taggable
{
  public static final String TYPE_ID = "LegacyUrlCapability";

  public static final CapabilityType TYPE = CapabilityType.capabilityType(TYPE_ID);

  @Override
  public CapabilityType type() {
    return TYPE;
  }

  @Override
  public String name() {
    return "Legacy style URLs";
  }

  @Override
  public Set<Tag> getTags() {
    return tags(categoryTag("Core"));
  }

  @Override
  public List<FormField> formFields() {
    return Collections.emptyList();
  }

  @Override
  protected LegacyUrlCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new LegacyUrlCapabilityConfiguration();
  }

  @Override
  public boolean isExposed() {
    return true;
  }

  @Override
  public boolean isHidden() {
    return false;
  }

  @Override
  protected String renderAbout() throws Exception {
    return render(TYPE_ID + "-about.vm");
  }
}
