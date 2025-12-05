
package org.globalart.astrum.internal.app;

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
import org.globalart.astrum.formfields.UrlFormField;

import com.google.common.collect.ImmutableList;

import static org.globalart.astrum.capability.Tag.categoryTag;
import static org.globalart.astrum.capability.Tag.tags;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * {@link BaseUrlCapabilityDescriptor} descriptor.
 *
 * @since 3.0
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(BaseUrlCapabilityDescriptor.TYPE_ID)
public class BaseUrlCapabilityDescriptor
    extends CapabilityDescriptorSupport<BaseUrlCapabilityConfiguration>
    implements Taggable
{
  public static final String TYPE_ID = "baseurl";

  public static final CapabilityType TYPE = CapabilityType.capabilityType(TYPE_ID);

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Base URL")
    String name();

    @DefaultMessage("URL")
    String urlLabel();

    @DefaultMessage("Reverse proxy base URL")
    String urlHelp();
  }

  private static final Messages messages = I18N.create(Messages.class);

  private final List<FormField> formFields;

  public BaseUrlCapabilityDescriptor() {
    this.formFields = ImmutableList.of(
        (FormField) new UrlFormField(
            BaseUrlCapabilityConfiguration.URL,
            messages.urlLabel(),
            messages.urlHelp(),
            FormField.MANDATORY));
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
  protected BaseUrlCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new BaseUrlCapabilityConfiguration(properties);
  }

  @Override
  protected String renderAbout() throws Exception {
    return render(TYPE_ID + "-about.vm");
  }
}
