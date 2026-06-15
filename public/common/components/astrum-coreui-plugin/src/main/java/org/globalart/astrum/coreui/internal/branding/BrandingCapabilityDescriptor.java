
package org.globalart.astrum.coreui.internal.branding;

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
import org.globalart.astrum.formfields.CheckboxFormField;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.TextAreaFormField;

import com.google.common.collect.Lists;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * {@link BrandingCapability} descriptor.
 *
 * @since 3.0
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(BrandingCapabilityDescriptor.TYPE_ID)
public class BrandingCapabilityDescriptor
    extends CapabilityDescriptorSupport<BrandingCapabilityConfiguration>
    implements Taggable
{
  public static final String TYPE_ID = "rapture.branding";

  public static final CapabilityType TYPE = CapabilityType.capabilityType(TYPE_ID);

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("UI: Branding")
    String name();

    @DefaultMessage("Enable header")
    String headerEnabledLabel();

    @DefaultMessage("Enable branding header HTML snippet.")
    String headerEnabledHelp();

    @DefaultMessage("Header HTML snippet")
    String headerHtmlLabel();

    @DefaultMessage("An HTML snippet to be included in branding header. Use '$baseUrl' to insert the base URL of the " +
        "server (e.g. to reference an image)")
    String headerHtmlHelp();

    @DefaultMessage("Enable footer")
    String footerEnabledLabel();

    @DefaultMessage("Enable branding footer HTML snippet.")
    String footerEnabledHelp();

    @DefaultMessage("Footer HTML snippet")
    String footerHtmlLabel();

    @DefaultMessage("An HTML snippet to be included in branding footer. Use '$baseUrl' to insert the base URL of the " +
        "server (e.g. to reference an image)")
    String footerHtmlHelp();
  }

  private static final Messages messages = I18N.create(Messages.class);

  private final List<FormField> formFields;

  public BrandingCapabilityDescriptor() {
    formFields = Lists.<FormField>newArrayList(
        new CheckboxFormField(
            BrandingCapabilityConfiguration.HEADER_ENABLED,
            messages.headerEnabledLabel(),
            messages.headerEnabledHelp(),
            FormField.OPTIONAL).withInitialValue(true),
        new TextAreaFormField(
            BrandingCapabilityConfiguration.HEADER_HTML,
            messages.headerHtmlLabel(),
            messages.headerHtmlHelp(),
            FormField.OPTIONAL),
        new CheckboxFormField(
            BrandingCapabilityConfiguration.FOOTER_ENABLED,
            messages.footerEnabledLabel(),
            messages.footerEnabledHelp(),
            FormField.OPTIONAL).withInitialValue(true),
        new TextAreaFormField(
            BrandingCapabilityConfiguration.FOOTER_HTML,
            messages.footerHtmlLabel(),
            messages.footerHtmlHelp(),
            FormField.OPTIONAL));
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
  protected BrandingCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new BrandingCapabilityConfiguration(properties);
  }

  @Override
  protected String renderAbout() throws Exception {
    return render(TYPE_ID + "-about.vm");
  }

  @Override
  public Set<Tag> getTags() {
    return Tag.tags(Tag.categoryTag("UI"));
  }

}
