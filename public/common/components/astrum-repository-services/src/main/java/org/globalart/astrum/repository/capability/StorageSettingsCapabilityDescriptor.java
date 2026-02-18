
package org.globalart.astrum.repository.capability;

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
import org.globalart.astrum.formfields.NumberTextFormField;

import com.google.common.collect.ImmutableList;

import static org.globalart.astrum.capability.Tag.categoryTag;
import static org.globalart.astrum.capability.Tag.tags;
import static org.globalart.astrum.formfields.FormField.MANDATORY;
import static org.globalart.astrum.repository.capability.GlobalRepositorySettings.DEFAULT_LAST_DOWNLOADED_INTERVAL;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * {@link StorageSettingsCapabilityDescriptor} descriptor.
 *
 * @since 3.16
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(StorageSettingsCapabilityDescriptor.TYPE_ID)
public class StorageSettingsCapabilityDescriptor
    extends CapabilityDescriptorSupport<StorageSettingsCapabilityConfiguration>
    implements Taggable
{
  public static final String TYPE_ID = "StorageSettings";

  public static final CapabilityType TYPE = CapabilityType.capabilityType(TYPE_ID);

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Storage Settings")
    String name();

    @DefaultMessage("Asset:")
    String assetLabel();

    @DefaultMessage("'Last Downloaded' Update Interval (hours)")
    String lastDownloadedIntervalLabel();
  }

  private static final Messages messages = I18N.create(Messages.class);

  private final List<FormField> formFields;

  public StorageSettingsCapabilityDescriptor() {
    this.formFields = ImmutableList.of(
        new NumberTextFormField(
            StorageSettingsCapabilityConfiguration.LAST_DOWNLOADED_INTERVAL,
            messages.assetLabel(),
            messages.lastDownloadedIntervalLabel(),
            MANDATORY)
                .withInitialValue(DEFAULT_LAST_DOWNLOADED_INTERVAL.toHours())
                .withMinimumValue(1));
  }

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
    return formFields;
  }

  @Override
  public Set<Tag> getTags() {
    return tags(categoryTag("Core"));
  }

  @Override
  protected StorageSettingsCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new StorageSettingsCapabilityConfiguration(properties);
  }

  @Override
  protected String renderAbout() throws Exception {
    return render(TYPE_ID + "-about.vm");
  }
}
