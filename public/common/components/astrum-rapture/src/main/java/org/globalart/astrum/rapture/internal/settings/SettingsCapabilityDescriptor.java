
package org.globalart.astrum.rapture.internal.settings;

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
import org.globalart.astrum.formfields.NumberTextFormField;
import org.globalart.astrum.formfields.StringTextFormField;
import org.globalart.astrum.rapture.settings.RaptureSettings;

import com.google.common.collect.Lists;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * {@link SettingsCapability} descriptor.
 *
 * @since 3.0
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(SettingsCapabilityDescriptor.TYPE_ID)
public class SettingsCapabilityDescriptor
    extends CapabilityDescriptorSupport<SettingsCapabilityConfiguration>
    implements Taggable
{
  public static final String TYPE_ID = "rapture.settings";

  public static final CapabilityType TYPE = CapabilityType.capabilityType(TYPE_ID);

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("UI: Settings")
    String name();

    @DefaultMessage("Debug allowed")
    String debugAllowedLabel();

    @DefaultMessage("Allow developer debugging")
    String debugAllowedHelp();

    @DefaultMessage("Authenticated user status interval")
    String statusIntervalAuthenticatedLabel();

    @DefaultMessage("Interval between status requests for authenticated users (seconds)")
    String statusIntervalAuthenticatedHelp();

    @DefaultMessage("Anonymous user status interval")
    String statusIntervalAnonymousLabel();

    @DefaultMessage("Interval between status requests for anonymous user (seconds)")
    String statusIntervalAnonymousHelp();

    @DefaultMessage("Session timeout")
    String sessionTimeoutLabel();

    @DefaultMessage("Period of inactivity before session times out (minutes). A value of 0 will mean that a session never expires.")
    String sessionTimeoutHelp();

    @DefaultMessage("Standard request timeout")
    String requestTimeoutLabel();

    @DefaultMessage("Period of time to keep the connection alive for requests expected to take a normal period of time (seconds)")
    String requestTimeoutHelp();

    @DefaultMessage("Extended request timeout")
    String longRequestTimeoutLabel();

    @DefaultMessage("Period of time to keep the connection alive for requests expected to take an extended period of time (seconds)")
    String longRequestTimeoutHelp();

    @DefaultMessage("Title")
    String titleLabel();

    @DefaultMessage("Browser page title")
    String titleHelp();
  }

  private static final Messages messages = I18N.create(Messages.class);

  private final List<FormField> formFields;

  public SettingsCapabilityDescriptor() {
    formFields = Lists.<FormField>newArrayList(
        new StringTextFormField(
            SettingsCapabilityConfiguration.TITLE,
            messages.titleLabel(),
            messages.titleHelp(),
            FormField.MANDATORY).withInitialValue(RaptureSettings.DEFAULT_TITLE),
        new CheckboxFormField(
            SettingsCapabilityConfiguration.DEBUG_ALLOWED,
            messages.debugAllowedLabel(),
            messages.debugAllowedHelp(),
            FormField.OPTIONAL).withInitialValue(RaptureSettings.DEFAULT_DEBUG_ALLOWED),
        new NumberTextFormField(
            SettingsCapabilityConfiguration.STATUS_INTERVAL_AUTHENTICATED,
            messages.statusIntervalAuthenticatedLabel(),
            messages.statusIntervalAuthenticatedHelp(),
            FormField.MANDATORY).withInitialValue(RaptureSettings.DEFAULT_STATUS_INTERVAL_AUTHENTICATED),
        new NumberTextFormField(
            SettingsCapabilityConfiguration.STATUS_INTERVAL_ANONYMOUS,
            messages.statusIntervalAnonymousLabel(),
            messages.statusIntervalAnonymousHelp(),
            FormField.MANDATORY).withInitialValue(RaptureSettings.DEFAULT_STATUS_INTERVAL_ANONYMOUS),
        new NumberTextFormField(
            SettingsCapabilityConfiguration.SESSION_TIMEOUT,
            messages.sessionTimeoutLabel(),
            messages.sessionTimeoutHelp(),
            FormField.MANDATORY).withInitialValue(RaptureSettings.DEFAULT_SESSION_TIMEOUT),
        new NumberTextFormField(
            SettingsCapabilityConfiguration.REQUEST_TIMEOUT,
            messages.requestTimeoutLabel(),
            messages.requestTimeoutHelp(),
            FormField.MANDATORY).withInitialValue(RaptureSettings.DEFAULT_REQUEST_TIMEOUT)
                .withMinimumValue(RaptureSettings.MIN_REQUEST_TIMEOUT),
        new NumberTextFormField(
            SettingsCapabilityConfiguration.LONG_REQUEST_TIMEOUT,
            messages.longRequestTimeoutLabel(),
            messages.longRequestTimeoutHelp(),
            FormField.MANDATORY).withInitialValue(RaptureSettings.DEFAULT_LONG_REQUEST_TIMEOUT)
                .withMinimumValue(RaptureSettings.DEFAULT_LONG_REQUEST_TIMEOUT));
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
  protected SettingsCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new SettingsCapabilityConfiguration(properties);
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
