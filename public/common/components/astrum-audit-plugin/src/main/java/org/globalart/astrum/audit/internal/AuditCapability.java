
package org.globalart.astrum.audit.internal;

import java.util.List;
import java.util.Map;
import java.util.Set;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.audit.AuditRecorder;
import org.globalart.astrum.audit.internal.AuditCapability.Configuration;
import org.globalart.astrum.capability.CapabilityConfigurationSupport;
import org.globalart.astrum.capability.CapabilityDescriptorSupport;
import org.globalart.astrum.capability.CapabilitySupport;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.capability.Tag;
import org.globalart.astrum.capability.Taggable;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.NumberTextFormField;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Collections.singleton;
import static org.globalart.astrum.capability.CapabilityType.capabilityType;

@Component(AuditCapability.TYPE_ID)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AuditCapability
    extends CapabilitySupport<Configuration>
{
  public static final String TYPE_ID = "audit";

  public static final CapabilityType TYPE = capabilityType(TYPE_ID);

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Audit")
    String name();

    @DefaultMessage("Audit")
    String category();

    @DefaultMessage("Disabled")
    String disabled();

    @DefaultMessage("Enabled")
    String enabled();

    @DefaultMessage("Retention (days)")
    String retentionDaysLabel();

    @DefaultMessage("Number of days audit events are retained before pruning. Default 90; minimum 1; maximum %s.")
    String retentionDaysHelp(int max);
  }

  static final Messages messages = I18N.create(Messages.class);

  private final AuditRecorder auditRecorder;

  private final AuditRetentionSettings retentionSettings;

  @Autowired
  public AuditCapability(final AuditRecorder auditRecorder, final AuditRetentionSettings retentionSettings) {
    this.auditRecorder = checkNotNull(auditRecorder);
    this.retentionSettings = checkNotNull(retentionSettings);
  }

  @Override
  protected Configuration createConfig(final Map<String, String> properties) {
    return new Configuration(properties, retentionSettings.getMaxRetentionDays());
  }

  @Override
  @Nullable
  protected String renderDescription() {
    if (context().isActive()) {
      return messages.enabled();
    }
    return messages.disabled();
  }

  @Override
  public Condition activationCondition() {
    return conditions().capabilities().passivateCapabilityDuringUpdate();
  }

  @Override
  protected void onActivate(final Configuration config) {
    if (auditRecorder instanceof AuditRecorderImpl) {
      ((AuditRecorderImpl) auditRecorder).setEnabled(true);
    }
    retentionSettings.setRetentionDays(config.getRetentionDays());
  }

  @Override
  protected void onUpdate(final Configuration config) {
    retentionSettings.setRetentionDays(config.getRetentionDays());
  }

  @Override
  protected void onPassivate(final Configuration config) {
    if (auditRecorder instanceof AuditRecorderImpl) {
      ((AuditRecorderImpl) auditRecorder).setEnabled(false);
    }
    // Intentionally does not reset retentionSettings: the cleanup task runs independently of the
    // capability lifecycle so existing rows continue to be pruned with the last-known retention.
  }

  public static class Configuration
      extends CapabilityConfigurationSupport
  {
    public static final String RETENTION_DAYS = "retentionDays";

    private final int retentionDays;

    public Configuration(final Map<String, String> properties, final int maxRetentionDays) {
      checkNotNull(properties);
      String raw = properties.get(RETENTION_DAYS);
      int parsed;
      try {
        parsed = parseInteger(raw, AuditRetentionSettings.DEFAULT_RETENTION_DAYS);
      }
      catch (NumberFormatException e) {
        throw new IllegalArgumentException(
            "retentionDays must be an integer between 1 and " + maxRetentionDays + ", got '" + raw + "'",
            e);
      }
      if (parsed < 1 || parsed > maxRetentionDays) {
        throw new IllegalArgumentException(
            "retentionDays must be between 1 and " + maxRetentionDays + ", got " + parsed);
      }
      this.retentionDays = parsed;
    }

    public int getRetentionDays() {
      return retentionDays;
    }

    @Override
    public String toString() {
      return "Configuration{retentionDays=" + retentionDays + '}';
    }
  }

  @AvailabilityVersion(from = "1.0")
  @Component
  @Qualifier(AuditCapability.TYPE_ID)
  public static class Descriptor
      extends CapabilityDescriptorSupport<Configuration>
      implements Taggable
  {
    private final AuditRetentionSettings retentionSettings;

    @Autowired
    public Descriptor(final AuditRetentionSettings retentionSettings) {
      this.retentionSettings = checkNotNull(retentionSettings);
      setExposed(true);
      setHidden(false);
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
      int max = retentionSettings.getMaxRetentionDays();
      return List.of(
          new NumberTextFormField(
              Configuration.RETENTION_DAYS,
              messages.retentionDaysLabel(),
              messages.retentionDaysHelp(max),
              FormField.MANDATORY)
                  .withInitialValue(AuditRetentionSettings.DEFAULT_RETENTION_DAYS)
                  .withMinimumValue(1)
                  .withMaximumValue(max));
    }

    @Override
    protected Configuration createConfig(final Map<String, String> properties) {
      return new Configuration(properties, retentionSettings.getMaxRetentionDays());
    }

    @Override
    protected String renderAbout() {
      return render(TYPE_ID + "-about.vm");
    }

    @Override
    public Set<Tag> getTags() {
      return singleton(Tag.categoryTag(messages.category()));
    }
  }
}
