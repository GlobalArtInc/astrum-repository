
package org.globalart.astrum.internal.scheduling;

import java.util.Collections;
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

import static org.globalart.astrum.capability.CapabilityType.capabilityType;
import static org.globalart.astrum.capability.Tag.categoryTag;
import static org.globalart.astrum.capability.Tag.tags;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * {@link SchedulerCapability} descriptor.
 *
 * @since 3.0
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(SchedulerCapabilityDescriptor.TYPE_ID)
public class SchedulerCapabilityDescriptor
    extends CapabilityDescriptorSupport<SchedulerCapabilityConfiguration>
    implements Taggable
{
  public static final String TYPE_ID = "scheduling.scheduler";

  public static final CapabilityType TYPE = capabilityType(TYPE_ID);

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Scheduler")
    String name();
  }

  private static final Messages messages = I18N.create(Messages.class);

  @Override
  public CapabilityType type() {
    return TYPE;
  }

  @Override
  public String name() {
    return messages.name();
  }

  @Override
  public List<FormField> formFields() {
    return Collections.emptyList();
  }

  @Override
  protected SchedulerCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new SchedulerCapabilityConfiguration(properties);
  }

  @Override
  protected String renderAbout() throws Exception {
    return render(TYPE_ID + "-about.vm");
  }

  @Override
  public Set<Tag> getTags() {
    return tags(categoryTag("Scheduling"));
  }
}
