
package org.globalart.astrum.onboarding.capability;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.capability.CapabilityDescriptorSupport;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.capability.Tag;
import org.globalart.astrum.capability.Taggable;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.FormField;

import static java.util.Collections.singletonList;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(OnboardingCapability.TYPE_ID)
public class OnboardingCapabilityDescriptor
    extends CapabilityDescriptorSupport<OnboardingCapabilityConfiguration>
    implements Taggable
{
  static final Messages messages = I18N.create(Messages.class);

  public OnboardingCapabilityDescriptor() {
    setExposed(false);
    setHidden(true);
  }

  @Override
  public CapabilityType type() {
    return OnboardingCapability.TYPE;
  }

  @Override
  public String name() {
    return messages.name();
  }

  @Override
  public List<FormField> formFields() {
    return new ArrayList<>();
  }

  @Override
  public Set<Tag> getTags() {
    return new HashSet<>(singletonList(Tag.categoryTag(messages.category())));
  }

  @Override
  protected OnboardingCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new OnboardingCapabilityConfiguration(properties);
  }

  interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Onboarding wizard")
    String name();

    @DefaultMessage("Onboarding")
    String category();

    @DefaultMessage("Enabled")
    String enabled();

    @DefaultMessage("Disabled")
    String disabled();
  }
}
