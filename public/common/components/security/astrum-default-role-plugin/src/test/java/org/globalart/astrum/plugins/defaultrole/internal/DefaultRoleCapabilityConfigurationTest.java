
package org.globalart.astrum.plugins.defaultrole.internal;

import java.lang.reflect.Field;
import java.util.Arrays;
import java.util.List;
import java.util.Map;

import jakarta.validation.groups.Default;

import org.globalart.astrum.security.role.RoleExistsString;
import org.globalart.astrum.validation.group.Create;
import org.globalart.astrum.validation.group.CreateNonExposed;
import org.globalart.astrum.validation.group.Load;
import org.globalart.astrum.validation.group.Update;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * Tests that {@link DefaultRoleCapabilityConfiguration} validation groups are correctly scoped so
 * that existence checks do not fire during capability LOAD (startup), preventing startup failures
 * when the referenced role is missing. See ASTRUM-52998.
 */
public class DefaultRoleCapabilityConfigurationTest
{
  @Test
  public void roleExistsConstraint_isNotInDefaultOrLoadGroup() throws Exception {
    Field roleField = DefaultRoleCapabilityConfiguration.class.getDeclaredField("role");
    RoleExistsString annotation = roleField.getAnnotation(RoleExistsString.class);

    assertThat(annotation).isNotNull();

    List<Class<?>> groups = Arrays.asList(annotation.groups());

    // Must NOT be in Default or Load groups — these fire during startup LOAD validation
    assertThat(groups).doesNotContain(Default.class);
    assertThat(groups).doesNotContain(Load.class);
  }

  @Test
  public void roleExistsConstraint_isInCreateAndUpdateGroups() throws Exception {
    Field roleField = DefaultRoleCapabilityConfiguration.class.getDeclaredField("role");
    RoleExistsString annotation = roleField.getAnnotation(RoleExistsString.class);

    assertThat(annotation).isNotNull();

    List<Class<?>> groups = Arrays.asList(annotation.groups());

    // Must fire during user-initiated create/update operations
    assertThat(groups).contains(Create.class, CreateNonExposed.class, Update.class);
  }

  @Test
  public void configuration_storesRoleFromProperties() {
    Map<String, String> props = Map.of(DefaultRoleCapabilityConfiguration.P_ROLE, "as-admin");

    DefaultRoleCapabilityConfiguration config = new DefaultRoleCapabilityConfiguration(props);

    assertThat(config.getRole()).isEqualTo("as-admin");
  }

  @Test
  public void configuration_handlesAbsentRole() {
    DefaultRoleCapabilityConfiguration config = new DefaultRoleCapabilityConfiguration(Map.of());

    assertThat(config.getRole()).isNull();
  }
}
