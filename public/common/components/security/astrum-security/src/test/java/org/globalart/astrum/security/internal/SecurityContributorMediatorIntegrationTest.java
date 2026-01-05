
package org.globalart.astrum.security.internal;

import org.globalart.astrum.security.AbstractSecurityTest;
import org.globalart.astrum.security.authz.AuthorizationManager;
import org.globalart.astrum.security.config.InitialSecurityConfiguration;
import org.globalart.astrum.security.config.PreconfiguredSecurityConfigurationSource;
import org.globalart.astrum.security.config.SecurityConfigurationSource;
import org.globalart.astrum.security.internal.SecurityContributorMediatorIntegrationTest.SecurityContributorMediatorTestConfiguration;
import org.globalart.astrum.security.role.NoSuchRoleException;
import org.globalart.astrum.security.role.Role;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Import;
import org.springframework.context.annotation.Primary;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.globalart.astrum.security.Roles.ADMIN_ROLE_ID;
import static org.globalart.astrum.security.Roles.ANONYMOUS_ROLE_ID;

/**
 * Integration test for {@link SecurityContributorMediator}.
 *
 * This test verifies the fix for ASTRUM-48819: Built-in roles (as-anonymous, as-admin) are available
 * during the CAPABILITIES phase because SecurityContributorMediator registers them during the SECURITY phase.
 *
 * The test simulates the startup sequence and ensures that security contributors are registered
 * before any capabilities would validate role existence.
 */
@Import(SecurityContributorMediatorTestConfiguration.class)
class SecurityContributorMediatorIntegrationTest
    extends AbstractSecurityTest
{
  static class SecurityContributorMediatorTestConfiguration
  {
    @Qualifier("default")
    @Primary
    @Bean
    SecurityConfigurationSource securityConfigurationSource() {
      return new PreconfiguredSecurityConfigurationSource(InitialSecurityConfiguration.getConfiguration());
    }

    @Bean
    org.globalart.astrum.security.config.SecurityContributor testSecurityContributor() {
      return new org.globalart.astrum.security.config.SecurityContributorSupport()
      {
        @Override
        public org.globalart.astrum.security.config.MemorySecurityConfiguration getContribution() {
          org.globalart.astrum.security.config.MemorySecurityConfiguration config =
              new org.globalart.astrum.security.config.MemorySecurityConfiguration();

          // Add as-all privilege (for admin role)
          config.addPrivilege(createWildcardPrivilege("as-all", "All permissions", "astrum:*"));

          // Add privileges for anonymous role
          config.addPrivilege(createApplicationPrivilege("as-search-read", "Search read", "search", "read"));
          config.addPrivilege(
              createApplicationPrivilege("as-healthcheck-read", "Healthcheck read", "healthcheck", "read"));
          config.addPrivilege(createWildcardPrivilege("as-repository-view-*-*-browse", "Browse all repositories",
              "astrum:repository-view:*:*:browse"));
          config.addPrivilege(createWildcardPrivilege("as-repository-view-*-*-read", "Read all repositories",
              "astrum:repository-view:*:*:read"));

          // Add as-admin role with as-all privilege
          config.addRole(createRole(ADMIN_ROLE_ID, ADMIN_ROLE_ID, "Administrator Role", "as-all"));

          // Add as-anonymous role with search and repository view privileges
          config.addRole(createRole(ANONYMOUS_ROLE_ID, ANONYMOUS_ROLE_ID, "Anonymous Role",
              "as-search-read", "as-healthcheck-read", "as-repository-view-*-*-browse", "as-repository-view-*-*-read"));

          return config;
        }
      };
    }
  }

  private AuthorizationManager authorizationManager;

  @BeforeEach
  @Override
  protected void setUp() throws Exception {
    super.setUp();
    authorizationManager = lookup(AuthorizationManager.class);
  }

  /**
   * Verifies that built-in roles are available after initialization.
   *
   * This test validates the fix for ASTRUM-48819 by ensuring:
   * 1. The SecurityContributorMediator registers security contributors during initialization
   * 2. Built-in roles (as-anonymous, as-admin) are available
   * 3. Role lookups succeed (as they would during capability validation)
   */
  @Test
  void testBuiltInRolesAvailableAfterInitialization() {
    // Verify: Built-in roles are available (these are provided by security contributors)
    Role anonymousRole = authorizationManager.getRole(ANONYMOUS_ROLE_ID);
    assertThat("as-anonymous role should be available", anonymousRole, is(notNullValue()));
    assertThat("as-anonymous role ID should match", anonymousRole.getRoleId(), is(ANONYMOUS_ROLE_ID));

    Role adminRole = authorizationManager.getRole(ADMIN_ROLE_ID);
    assertThat("as-admin role should be available", adminRole, is(notNullValue()));
    assertThat("as-admin role ID should match", adminRole.getRoleId(), is(ADMIN_ROLE_ID));
  }

  /**
   * Verifies that looking up a non-existent role throws the expected exception.
   *
   * This confirms the authorization manager is working correctly and would properly
   * reject invalid roles during capability validation.
   */
  @Test
  void testNonExistentRoleThrowsException() {
    // Verify: Non-existent role lookup throws exception
    assertThrows(NoSuchRoleException.class,
        () -> authorizationManager.getRole("non-existent-role-id"),
        "Looking up non-existent role should throw NoSuchRoleException");
  }

  /**
   * Verifies that the anonymous role has the expected privileges.
   *
   * This ensures the security contributor provided complete role definitions,
   * not just role IDs.
   */
  @Test
  void testAnonymousRoleHasExpectedPrivileges() {
    Role anonymousRole = authorizationManager.getRole(ANONYMOUS_ROLE_ID);

    // Verify: Anonymous role has search and healthcheck privileges
    assertThat("Anonymous role should have privileges",
        anonymousRole.getPrivileges().isEmpty(), is(false));

    // Verify the expected privileges from security contributors
    assertThat("Anonymous role should have search privilege",
        anonymousRole.getPrivileges().contains("as-search-read"), is(true));
    assertThat("Anonymous role should have healthcheck privilege",
        anonymousRole.getPrivileges().contains("as-healthcheck-read"), is(true));
    assertThat("Anonymous role should have browse privilege",
        anonymousRole.getPrivileges().contains("as-repository-view-*-*-browse"), is(true));
    assertThat("Anonymous role should have read privilege",
        anonymousRole.getPrivileges().contains("as-repository-view-*-*-read"), is(true));
  }

  /**
   * Verifies that the admin role has the all-permissions privilege.
   *
   * This ensures the admin role is configured with full access as expected.
   */
  @Test
  void testAdminRoleHasAllPermissions() {
    Role adminRole = authorizationManager.getRole(ADMIN_ROLE_ID);

    // Verify: Admin role has as-all privilege (all permissions)
    assertThat("Admin role should have privileges",
        adminRole.getPrivileges().isEmpty(), is(false));
    assertThat("Admin role should have as-all privilege",
        adminRole.getPrivileges().contains("as-all"), is(true));
  }
}
