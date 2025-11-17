package org.globalart.astrum.internal.security.model.upgrade;

import java.nio.charset.StandardCharsets;
import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import java.util.stream.Collectors;
import java.util.stream.StreamSupport;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.internal.security.model.CPrivilegeDAO;
import org.globalart.astrum.internal.security.model.CPrivilegeData;
import org.globalart.astrum.internal.security.model.CRoleDAO;
import org.globalart.astrum.internal.security.model.CRoleData;
import org.globalart.astrum.internal.security.model.CUserRoleMappingDAO;
import org.globalart.astrum.internal.security.model.CUserRoleMappingData;
import org.globalart.astrum.testdb.DataSessionRule;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;

public class SecurityIdPrefixMigrationStep_2_165Test
{
  private static final ObjectMapper OBJECT_MAPPER = new ObjectMapper();

  @Rule
  public DataSessionRule sessionRule = new DataSessionRule()
      .access(CRoleDAO.class)
      .access(CPrivilegeDAO.class)
      .access(CUserRoleMappingDAO.class);

  private final SecurityIdPrefixMigrationStep_2_165 underTest = new SecurityIdPrefixMigrationStep_2_165();

  @Before
  public void clearTables() throws Exception {
    try (Connection connection = sessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.runStatement(connection, "DELETE FROM role");
      underTest.runStatement(connection, "DELETE FROM privilege");
      underTest.runStatement(connection, "DELETE FROM user_role_mapping");
      underTest.runStatement(connection, "DROP TABLE IF EXISTS capability_storage_item");
    }
  }

  @Test
  public void testVersion() {
    assertThat(underTest.version(), is(Optional.of("2.165")));
  }

  @Test
  public void testRename() {
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("nx-admin"), is("as-admin"));
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("nx-anonymous"), is("as-anonymous"));
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("nx-all"), is("as-all"));
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("nx-metrics-all"), is("as-metrics-all"));
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("nx-users-read"), is("as-users-read"));
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("nx-ssl-truststore-delete"), is("as-ssl-truststore-delete"));
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("nx-tasks-run"), is("as-tasks-run"));
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("nx-repository-view-*-*-read"),
        is("as-repository-view-*-*-read"));
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("nx-repository-admin-maven2-maven-releases-edit"),
        is("as-repository-admin-maven2-maven-releases-edit"));
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("nx-repository-content-selector-cs-maven2-*-read"),
        is("as-repository-content-selector-cs-maven2-*-read"));
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("nx-script-*-run"), is("as-script-*-run"));
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("nx-users-custom"), is("nx-users-custom"));
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("nx-developers"), is("nx-developers"));
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("nx-repository-view-"), is("nx-repository-view-"));
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("deployer"), is("deployer"));
    assertThat(SecurityIdPrefixMigrationStep_2_165.rename("as-admin"), is("as-admin"));
  }

  @Test
  public void testMigrateRewritesStoredSecurityData() throws Exception {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      CRoleDAO roles = session.access(CRoleDAO.class);
      roles.create(role("deployer", "deployer",
          Set.of("nx-repository-view-*-*-read", "nx-search-read", "custom-priv", "nx-custom"),
          Set.of("nx-anonymous", "readers")));
      roles.create(role("readers", "readers", Set.of("nx-repository-view-maven2-releases-browse"), Set.of()));
      roles.create(role("untouched", "untouched", Set.of("custom-priv"), Set.of("readers")));
      roles.create(role("nx-admin", "nx-admin", Set.of("nx-all"), Set.of()));

      CPrivilegeDAO privileges = session.access(CPrivilegeDAO.class);
      String generated = "nx-repository-view-maven2-releases-read";
      privileges.create(privilege(generated, generated));
      privileges.create(privilege("nx-custom", "nx-custom"));
      privileges.create(privilege("custom-priv", "custom-priv"));

      CUserRoleMappingDAO mappings = session.access(CUserRoleMappingDAO.class);
      mappings.create(mapping("admin", "default", Set.of("nx-admin")));
      mappings.create(mapping("anonymous", "default", Set.of("nx-anonymous")));
      mappings.create(mapping("bob", "default", Set.of("nx-anonymous", "deployer", "nx-custom-role")));
      mappings.create(mapping("carol", "OIDC", Set.of("deployer")));
      session.getTransaction().commit();
    }

    try (Connection connection = sessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      createCapabilityTable(connection);
      insertCapability(connection, UUID.randomUUID(), "defaultrole", Map.of("role", "nx-anonymous"));
      insertCapability(connection, UUID.randomUUID(), "defaultrole", Map.of("role", "nx-custom-role"));
      insertCapability(connection, UUID.randomUUID(), "other", Map.of("role", "nx-anonymous"));

      underTest.migrate(connection);
      underTest.migrate(connection);

      assertThat(capabilityRoles(connection, "defaultrole"), containsInAnyOrder("as-anonymous", "nx-custom-role"));
      assertThat(capabilityRoles(connection, "other"), containsInAnyOrder("nx-anonymous"));
    }

    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      CRoleDAO roles = session.access(CRoleDAO.class);
      CRoleData deployer = roles.read("deployer").orElseThrow();
      assertThat(deployer.getPrivileges(),
          containsInAnyOrder("as-repository-view-*-*-read", "as-search-read", "custom-priv", "nx-custom"));
      assertThat(deployer.getRoles(), containsInAnyOrder("as-anonymous", "readers"));
      assertThat(roles.read("readers").orElseThrow().getPrivileges(),
          containsInAnyOrder("as-repository-view-maven2-releases-browse"));
      assertThat(roles.read("untouched").orElseThrow().getRoles(), containsInAnyOrder("readers"));
      assertThat(roles.read("nx-admin").isPresent(), is(false));
      CRoleData admin = roles.read("as-admin").orElseThrow();
      assertThat(admin.getName(), is("as-admin"));
      assertThat(admin.getPrivileges(), containsInAnyOrder("as-all"));

      CPrivilegeDAO privileges = session.access(CPrivilegeDAO.class);
      assertThat(ids(privileges.browse()),
          containsInAnyOrder("as-repository-view-maven2-releases-read", "nx-custom", "custom-priv"));
      assertThat(privileges.read("as-repository-view-maven2-releases-read").orElseThrow().getName(),
          is("as-repository-view-maven2-releases-read"));

      CUserRoleMappingDAO mappings = session.access(CUserRoleMappingDAO.class);
      assertThat(mappings.read("admin", "default").orElseThrow().getRoles(), containsInAnyOrder("as-admin"));
      assertThat(mappings.read("anonymous", "default").orElseThrow().getRoles(), containsInAnyOrder("as-anonymous"));
      assertThat(mappings.read("bob", "default").orElseThrow().getRoles(),
          containsInAnyOrder("as-anonymous", "deployer", "nx-custom-role"));
      assertThat(mappings.read("carol", "OIDC").orElseThrow().getRoles(), containsInAnyOrder("deployer"));
    }
  }

  @Test
  public void testMigrateMergesIntoExistingRenamedRole() throws Exception {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      CRoleDAO roles = session.access(CRoleDAO.class);
      roles.create(role("nx-anonymous", "nx-anonymous", Set.of("nx-search-read"), Set.of("readers")));
      roles.create(role("as-anonymous", "as-anonymous", Set.of("as-healthcheck-read"), Set.of("nx-anonymous")));
      roles.create(role("readers", "readers", Set.of(), Set.of()));
      CPrivilegeDAO privileges = session.access(CPrivilegeDAO.class);
      privileges.create(privilege("nx-script-*-run", "nx-script-*-run"));
      privileges.create(privilege("as-script-*-run", "as-script-*-run"));
      session.getTransaction().commit();
    }

    try (Connection connection = sessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.migrate(connection);
    }

    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      CRoleDAO roles = session.access(CRoleDAO.class);
      assertThat(ids(roles.browse()), containsInAnyOrder("as-anonymous", "readers"));
      CRoleData anonymous = roles.read("as-anonymous").orElseThrow();
      assertThat(anonymous.getPrivileges(), containsInAnyOrder("as-search-read", "as-healthcheck-read"));
      assertThat(anonymous.getRoles(), containsInAnyOrder("readers"));
      assertThat(ids(session.access(CPrivilegeDAO.class).browse()), containsInAnyOrder("as-script-*-run"));
    }
  }

  @Test
  public void testMigrateWithoutTables() throws Exception {
    try (Connection connection = sessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.runStatement(connection, "DROP TABLE IF EXISTS role");
      underTest.runStatement(connection, "DROP TABLE IF EXISTS privilege");
      underTest.runStatement(connection, "DROP TABLE IF EXISTS user_role_mapping");

      underTest.migrate(connection);

      assertThat(underTest.tableExists(connection, "role"), is(false));
    }
  }

  private static CRoleData role(
      final String id,
      final String name,
      final Set<String> privileges,
      final Set<String> roles)
  {
    CRoleData role = new CRoleData();
    role.setId(id);
    role.setName(name);
    role.setDescription(name);
    role.setPrivileges(privileges);
    role.setRoles(roles);
    return role;
  }

  private static CPrivilegeData privilege(final String id, final String name) {
    CPrivilegeData privilege = new CPrivilegeData();
    privilege.setId(id);
    privilege.setName(name);
    privilege.setDescription(name);
    privilege.setType("application");
    privilege.setProperties(Map.of("domain", "custom", "actions", "read"));
    return privilege;
  }

  private static CUserRoleMappingData mapping(final String userId, final String source, final Set<String> roles) {
    CUserRoleMappingData mapping = new CUserRoleMappingData();
    mapping.setUserId(userId);
    mapping.setSource(source);
    mapping.setRoles(roles);
    return mapping;
  }

  private static List<String> ids(final Iterable<?> entities) {
    return StreamSupport.stream(entities.spliterator(), false)
        .map(entity -> entity instanceof CRoleData role ? role.getId() : ((CPrivilegeData) entity).getId())
        .collect(Collectors.toList());
  }

  private void createCapabilityTable(final Connection connection) throws Exception {
    String json = underTest.isH2(connection) ? "JSON" : "JSONB";
    underTest.runStatement(connection, """
        CREATE TABLE IF NOT EXISTS capability_storage_item (
          id         UUID         NOT NULL,
          version    INT          NOT NULL,
          type       VARCHAR(100) NOT NULL,
          enabled    BOOLEAN      NOT NULL,
          notes      VARCHAR(400) NULL,
          properties %s NOT NULL,
          CONSTRAINT pk_capability_storage_item_id PRIMARY KEY (id)
        )
        """.formatted(json));
  }

  private void insertCapability(
      final Connection connection,
      final UUID id,
      final String type,
      final Map<String, String> properties) throws Exception
  {
    try (PreparedStatement statement = connection.prepareStatement(
        "INSERT INTO capability_storage_item (id, version, type, enabled, properties) VALUES (?, 1, ?, TRUE, ?)")) {
      statement.setObject(1, id);
      statement.setString(2, type);
      underTest.setJsonParameter(statement, 3, OBJECT_MAPPER.writeValueAsBytes(properties), underTest.isH2(connection));
      statement.executeUpdate();
    }
  }

  private static List<String> capabilityRoles(final Connection connection, final String type) throws Exception {
    try (PreparedStatement statement = connection.prepareStatement(
        "SELECT properties FROM capability_storage_item WHERE type = ?")) {
      statement.setString(1, type);
      try (ResultSet rs = statement.executeQuery()) {
        List<String> roles = new ArrayList<>();
        while (rs.next()) {
          Map<?, ?> properties =
              OBJECT_MAPPER.readValue(new String(rs.getBytes(1), StandardCharsets.UTF_8), Map.class);
          roles.add((String) properties.get("role"));
        }
        return roles;
      }
    }
  }
}
