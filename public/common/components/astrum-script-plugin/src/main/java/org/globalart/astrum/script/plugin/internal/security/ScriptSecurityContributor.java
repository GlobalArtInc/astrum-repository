
package org.globalart.astrum.script.plugin.internal.security;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;
import org.globalart.astrum.security.config.memory.MemoryCPrivilege;
import org.globalart.astrum.security.config.memory.MemoryCPrivilege.MemoryCPrivilegeBuilder;

import static org.apache.commons.lang3.StringUtils.capitalize;
import static org.globalart.astrum.script.plugin.internal.security.ScriptPrivilegeDescriptor.P_ACTIONS;
import static org.globalart.astrum.script.plugin.internal.security.ScriptPrivilegeDescriptor.P_NAME;
import static org.globalart.astrum.script.plugin.internal.security.ScriptPrivilegeDescriptor.TYPE;
import static org.globalart.astrum.security.privilege.PrivilegeDescriptorSupport.ALL;
import org.springframework.stereotype.Component;

/**
 * Script security configuration.
 *
 * @since 3.0
 */
@Component
public class ScriptSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  public static final String SCRIPT_ALL_PREFIX = "as-script-*-";

  public static final String SCRIPT_ALL_DESCRIPTION_SUFFIX = " permissions for Scripts";

  public static final String ACTION_BROWSE = "browse";

  public static final String ACTION_READ = "read";

  public static final String ACTION_EDIT = "edit";

  public static final String ACTION_ADD = "add";

  public static final String ACTION_DELETE = "delete";

  public static final String ACTION_RUN = "run";

  @Override
  public SecurityConfiguration getContribution() {
    MemorySecurityConfiguration config = new MemorySecurityConfiguration();

    config.addPrivilege(createScriptPrivilege(ALL));
    config.addPrivilege(createScriptPrivilege(ACTION_BROWSE));
    config.addPrivilege(createScriptPrivilege(ACTION_READ));
    config.addPrivilege(createScriptPrivilege(ACTION_EDIT));
    config.addPrivilege(createScriptPrivilege(ACTION_ADD));
    config.addPrivilege(createScriptPrivilege(ACTION_DELETE));
    config.addPrivilege(createScriptPrivilege(ACTION_RUN));

    return config;
  }

  private MemoryCPrivilege createScriptPrivilege(
      final String action)
  {
    final String id = SCRIPT_ALL_PREFIX + action;
    final String description = (ALL.equals(action) ? "All" : capitalize(action)) + SCRIPT_ALL_DESCRIPTION_SUFFIX;
    final String actions =
        (ALL.equals(action) || ACTION_READ.equals(action) || ACTION_RUN.equals(action))
            ? action
            : (String.join(",",
                action, ACTION_READ));

    return new MemoryCPrivilegeBuilder(id)
        .type(TYPE)
        .readOnly(true)
        .name(id)
        .description(description)
        .property(P_NAME, ALL)
        .property(P_ACTIONS, actions)
        .build();
  }
}
