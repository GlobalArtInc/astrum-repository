
package org.globalart.astrum.plugins.defaultrole.internal;

import java.util.List;
import java.util.Map;
import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.capability.CapabilityDescriptorSupport;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.ComboboxFormField;
import org.globalart.astrum.formfields.FormField;

import static java.util.Collections.singletonList;
import static org.globalart.astrum.capability.CapabilityType.capabilityType;
import static org.globalart.astrum.plugins.defaultrole.internal.DefaultRoleCapabilityConfiguration.P_ROLE;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Descriptor of the {@link DefaultRoleCapability} for UI configuration
 *
 * @since 3.22
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(DefaultRoleCapabilityDescriptor.TYPE_ID)
public class DefaultRoleCapabilityDescriptor
    extends CapabilityDescriptorSupport<DefaultRoleCapabilityConfiguration>
{
  public static final String TYPE_ID = "defaultrole";

  public static final CapabilityType TYPE = capabilityType(TYPE_ID);

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Default Role")
    String name();

    @DefaultMessage("Role")
    String roleLabel();

    @DefaultMessage("The role which is automatically granted to authenticated users")
    String roleHelp();
  }

  private static final Messages messages = I18N.create(Messages.class);

  private final ComboboxFormField role;

  public DefaultRoleCapabilityDescriptor() {
    setExposed(true);
    setHidden(false);

    this.role = new ComboboxFormField<String>(
        P_ROLE,
        messages.roleLabel(),
        messages.roleHelp(),
        FormField.MANDATORY).withStoreApi("coreui_Role.read");
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
    return singletonList(role);
  }

  @Override
  protected String renderAbout() {
    return render(TYPE_ID + "-about.vm");
  }

  @Override
  protected DefaultRoleCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new DefaultRoleCapabilityConfiguration(properties);
  }
}
