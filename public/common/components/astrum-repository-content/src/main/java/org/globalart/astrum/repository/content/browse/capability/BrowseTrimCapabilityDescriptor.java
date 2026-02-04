
package org.globalart.astrum.repository.content.browse.capability;

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
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(BrowseTrimCapabilityDescriptor.TYPE_ID)
public class BrowseTrimCapabilityDescriptor
    extends CapabilityDescriptorSupport<BrowseTrimCapabilityConfiguration>
    implements Taggable
{
  public static final String TYPE_ID = "browse.trim";

  public static final CapabilityType TYPE = CapabilityType.capabilityType(TYPE_ID);

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Repository: Browse Trim")
    String name();

    @DefaultMessage("<p>Controls automatic trimming of empty browse nodes (folders) in repository browse trees. " +
        "PostgreSQL databases have trim disabled by default for performance reasons. " +
        "H2 databases always have trim enabled.</p>" +
        "<p><strong>Configuration Options:</strong></p>" +
        "<ul>" +
        "<li><strong>Enable PostgreSQL Trim:</strong> When enabled, empty browse nodes are automatically removed after component deletion. WARNING: This can cause performance issues on large databases.</li>"
        +
        "<li><strong>Enable Batch Trim Processing:</strong> Overrides 'astrum.browse.rebuild.noPurgeDelay' property. When enabled, trim operations are batched (every 2 seconds or 100 events) to reduce database contention. Improves performance on databases with frequent deletions. NOTE: Trim operations may be delayed up to 2 seconds.</li>"
        +
        "</ul>")
    String about();

    @DefaultMessage("Enable PostgreSQL trim (PostgreSQL only)")
    String postgresqlTrimEnabledLabel();

    @DefaultMessage("[PostgreSQL only] Enable automatic trimming of empty browse nodes. " +
        "WARNING: This can cause performance issues on large databases. " +
        "Recommended to enable unless you have a very large database. " +
        "NOTE: H2 databases always have trim enabled and are not affected by this setting.")
    String postgresqlTrimEnabledHelp();

    @DefaultMessage("Enable batch trim processing (All databases)")
    String batchTrimEnabledLabel();

    @DefaultMessage("[Applies to all databases] This capability overrides the 'astrum.browse.rebuild.noPurgeDelay' property. "
        +
        "When enabled, trim operations are batched (processed every 2 seconds or 100 events) to reduce database contention. "
        +
        "This can improve performance on large databases with frequent deletions. " +
        "When disabled, trim operations happen immediately after each component/asset deletion (default behavior). " +
        "NOTE: When batch processing is enabled, trim operations will not execute immediately and may be delayed up to 2 seconds.")
    String batchTrimEnabledHelp();
  }

  private static final Messages messages = I18N.create(Messages.class);

  private final List<FormField> formFields;

  public BrowseTrimCapabilityDescriptor() {
    formFields = List.of(
        new CheckboxFormField(
            BrowseTrimCapabilityConfiguration.POSTGRESQL_TRIM_ENABLED,
            messages.postgresqlTrimEnabledLabel(),
            messages.postgresqlTrimEnabledHelp(),
            FormField.OPTIONAL).withInitialValue(true),
        new CheckboxFormField(
            BrowseTrimCapabilityConfiguration.BATCH_TRIM_ENABLED,
            messages.batchTrimEnabledLabel(),
            messages.batchTrimEnabledHelp(),
            FormField.OPTIONAL).withInitialValue(false));
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
  public String about() {
    return messages.about();
  }

  @Override
  public List<FormField> formFields() {
    return formFields;
  }

  @Override
  protected BrowseTrimCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new BrowseTrimCapabilityConfiguration(properties);
  }

  @Override
  public Set<Tag> getTags() {
    return Tag.tags(Tag.categoryTag("Repository"));
  }

  @Override
  public boolean isExposed() {
    return true;
  }
}
