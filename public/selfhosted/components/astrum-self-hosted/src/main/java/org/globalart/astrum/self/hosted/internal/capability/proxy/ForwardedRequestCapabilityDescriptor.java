
package org.globalart.astrum.self.hosted.internal.capability.proxy;

import java.util.List;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.capability.CapabilityDescriptorSupport;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.capability.Tag;
import org.globalart.astrum.capability.Taggable;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.CheckboxFormField;
import org.globalart.astrum.formfields.FormField;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static java.util.Collections.singleton;

/**
 * Descriptor for ForwardedRequestCapability.
 */
@AvailabilityVersion(from = ForwardedRequestCapabilityUpgradeStep_2_131.VERSION)
@Component
@Qualifier(ForwardedRequestCapability.TYPE_ID)
public class ForwardedRequestCapabilityDescriptor
    extends CapabilityDescriptorSupport<ForwardedRequestCapabilityConfiguration>
    implements Taggable
{
  private final List<FormField> formFields;

  @Autowired
  public ForwardedRequestCapabilityDescriptor() {
    formFields = List.of(
        new CheckboxFormField(
            "enabled",
            "Enable processing of forwarded headers",
            "When enabled, X-Forwarded-For, X-Forwarded-Proto and related headers will be processed. "
                + "When disabled, the original connection information is preserved.",
            FormField.OPTIONAL).withInitialValue(true));
    setExposed(true);
    setHidden(false);
  }

  @Override
  public CapabilityType type() {
    return ForwardedRequestCapability.TYPE;
  }

  @Override
  public String name() {
    return ForwardedRequestCapability.messages.name();
  }

  @Override
  public List<FormField> formFields() {
    return formFields;
  }

  @Override
  protected ForwardedRequestCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new ForwardedRequestCapabilityConfiguration(properties);
  }

  @Override
  protected String renderAbout() {
    return render(ForwardedRequestCapability.TYPE_ID + "-about.vm");
  }

  @Override
  public Set<Tag> getTags() {
    return singleton(Tag.categoryTag(ForwardedRequestCapability.messages.category()));
  }
}
