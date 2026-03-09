
package org.globalart.astrum.internal.script;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.StringTextFormField;
import org.globalart.astrum.formfields.TextAreaFormField;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * {@link ScriptTask} descriptor.
 *
 * @since 3.0
 */
@AvailabilityVersion(from = "1.0")
@Component
public class ScriptTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "script";

  public static final String LANGUAGE = "language";

  public static final String SOURCE = "source";

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Admin - Execute script")
    String name();

    @DefaultMessage("Language")
    String languageLabel();

    @DefaultMessage("Script language")
    String languageHelpText();

    @DefaultMessage("Source")
    String sourceLabel();

    @DefaultMessage("Script source")
    String sourceHelpText();
  }

  private static final Messages messages = I18N.create(Messages.class);

  // TODO: validate language is registered? Maybe use combo to only allow selection of valid languages?

  // TODO: validate source is valid? This may not be easy to do or reliable w/o actually executing?

  // TODO: this task may expose a lot of potential for misuse, and may need to be optional enabled by system property

  @Autowired
  public ScriptTaskDescriptor(
      final NodeAccess nodeAccess,
      @Value("${astrum.scripts.allowCreation:false}") final boolean allowCreation)
  {
    super(TYPE_ID,
        ScriptTask.class,
        messages.name(),
        TaskDescriptorSupport.VISIBLE,
        isExposed(allowCreation),
        new StringTextFormField(
            LANGUAGE,
            messages.languageLabel(),
            messages.languageHelpText(),
            FormField.MANDATORY).withInitialValue(ScriptEngineManagerProvider.DEFAULT_LANGUAGE),
        new TextAreaFormField(
            SOURCE,
            messages.sourceLabel(),
            messages.sourceHelpText(),
            FormField.MANDATORY,
            null,
            !allowCreation),
        nodeAccess.isClustered() ? TaskDescriptorSupport.newMultinodeFormField() : null);
  }

  /**
   * If the allowCreation flag is false we don't want this task exposed to user, but still want
   * existing scripts runnable
   */
  private static boolean isExposed(boolean allowCreation) {
    return allowCreation;
  }
}
