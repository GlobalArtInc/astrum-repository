

package org.globalart.astrum.blobstore.s3.internal.capability;

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
import org.globalart.astrum.formfields.StringTextFormField;

import com.google.common.collect.ImmutableList;

import static org.globalart.astrum.capability.Tag.categoryTag;
import static org.globalart.astrum.capability.Tag.tags;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

@Component
@Qualifier(CustomS3RegionCapabilityDescriptor.TYPE_ID)
@AvailabilityVersion(from = "2.4")
public class CustomS3RegionCapabilityDescriptor
    extends CapabilityDescriptorSupport<CustomS3RegionCapabilityConfiguration>
    implements Taggable
{
  public static final String TYPE_ID = "customs3regions";

  public static final CapabilityType TYPE_NAME = CapabilityType.capabilityType(TYPE_ID);

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Custom S3 Regions")
    String name();

    @DefaultMessage("Regions")
    String regionsLabel();

    @DefaultMessage("Custom S3 Regions, separated by commas")
    String regionsHelp();
  }

  private static final Messages messages = I18N.create(Messages.class);

  @SuppressWarnings("rawtypes")
  private final List<FormField> formFields;

  public CustomS3RegionCapabilityDescriptor() {
    formFields = ImmutableList.of(
        new StringTextFormField(
            CustomS3RegionCapabilityConfiguration.REGIONS,
            messages.regionsLabel(),
            messages.regionsHelp(),
            FormField.MANDATORY));
  }

  @Override
  public CapabilityType type() {
    return TYPE_NAME;
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
    return tags(categoryTag("S3"));
  }

  @Override
  protected CustomS3RegionCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new CustomS3RegionCapabilityConfiguration(properties);
  }

  @Override
  protected String renderAbout() throws Exception {
    return render(TYPE_ID + "-about.vm");
  }
}
