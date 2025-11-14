package org.globalart.astrum.internal.security.model.upgrade;

import java.io.IOException;
import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.ArrayList;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

@Component
public class SecurityIdPrefixMigrationStep_2_165
    implements DatabaseMigrationStep
{
  static final String OLD_PREFIX = "nx-";

  static final String NEW_PREFIX = "as-";

  private static final Set<String> BUILT_IN_IDS = Set.of(
      "admin",
      "anonymous",
      "all",
      "settings-all",
      "settings-read",
      "settings-update",
      "bundles-all",
      "bundles-read",
      "search-read",
      "apikey-all",
      "userschangepw",
      "component-upload",
      "healthcheck-read",
      "metrics-all",
      "logging-all",
      "logging-read",
      "logging-update",
      "logging-mark",
      "audit-all",
      "audit-read",
      "wonderland-all",
      "analytics-all",
      "atlas-all",
      "tasks-run",
      "uploader-metadata-read");

  private static final List<String> CRUD_ID_BASES = List.of(
      "privileges",
      "roles",
      "users",
      "capabilities",
      "tasks",
      "selectors",
      "datastores",
      "blobstores",
      "ssl-truststore");

  private static final List<String> CRUD_ACTIONS = List.of("all", "create", "read", "update", "delete");

  private static final List<String> GENERATED_ID_PREFIXES = List.of(
      "repository-view-",
      "repository-admin-",
      "repository-content-selector-",
      "script-");

  private static final String DEFAULT_ROLE_CAPABILITY_TYPE = "defaultrole";

  private static final String DEFAULT_ROLE_PROPERTY = "role";

  private static final TypeReference<List<String>> STRING_LIST = new TypeReference<>() { };

  private static final TypeReference<LinkedHashMap<String, Object>> PROPERTIES_MAP = new TypeReference<>() { };

  private static final ObjectMapper OBJECT_MAPPER = new ObjectMapper();

  private final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Optional<String> version() {
    return Optional.of("2.165");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    boolean h2 = isH2(connection);
    if (tableExists(connection, "privilege")) {
      migratePrivileges(connection);
    }
    if (tableExists(connection, "role")) {
      migrateRoles(connection, h2);
    }
    if (tableExists(connection, "user_role_mapping")) {
      migrateUserRoleMappings(connection, h2);
    }
    if (tableExists(connection, "capability_storage_item")) {
      migrateDefaultRoleCapabilities(connection, h2);
    }
  }

  static String rename(final String id) {
    if (id == null || !id.startsWith(OLD_PREFIX)) {
      return id;
    }
    String rest = id.substring(OLD_PREFIX.length());
    return isBuiltIn(rest) ? NEW_PREFIX + rest : id;
  }

  private static boolean isBuiltIn(final String rest) {
    if (BUILT_IN_IDS.contains(rest)) {
      return true;
    }
    for (String base : CRUD_ID_BASES) {
      for (String action : CRUD_ACTIONS) {
        if (rest.equals(base + "-" + action)) {
          return true;
        }
      }
    }
    for (String prefix : GENERATED_ID_PREFIXES) {
      if (rest.startsWith(prefix) && rest.length() > prefix.length()) {
        return true;
      }
    }
    return false;
  }

  private void migratePrivileges(final Connection connection) throws SQLException {
    Map<String, String> names = new LinkedHashMap<>();
    try (PreparedStatement select = connection.prepareStatement("SELECT id, name FROM privilege");
         ResultSet rs = select.executeQuery()) {
      while (rs.next()) {
        names.put(rs.getString("id"), rs.getString("name"));
      }
    }
    int renamed = 0;
    int removed = 0;
    for (Map.Entry<String, String> entry : names.entrySet()) {
      String id = entry.getKey();
      String newId = rename(id);
      if (newId.equals(id)) {
        continue;
      }
      if (names.containsKey(newId)) {
        try (PreparedStatement delete = connection.prepareStatement("DELETE FROM privilege WHERE id = ?")) {
          delete.setString(1, id);
          delete.executeUpdate();
        }
        removed++;
        continue;
      }
      String name = entry.getValue();
      try (PreparedStatement update =
          connection.prepareStatement("UPDATE privilege SET id = ?, name = ? WHERE id = ?")) {
        update.setString(1, newId);
        update.setString(2, id.equals(name) ? newId : name);
        update.setString(3, id);
        update.executeUpdate();
      }
      renamed++;
    }
    log.info("Renamed {} stored privileges to the {} prefix, removed {} duplicates", renamed, NEW_PREFIX, removed);
  }

  private void migrateRoles(final Connection connection, final boolean h2) throws SQLException, IOException {
    Map<String, RoleRow> rows = new LinkedHashMap<>();
    try (PreparedStatement select = connection.prepareStatement("SELECT id, name, privileges, roles FROM role");
         ResultSet rs = select.executeQuery()) {
      while (rs.next()) {
        RoleRow row = new RoleRow(rs.getString("id"), rs.getString("name"),
            readList(rs.getBytes("privileges")), readList(rs.getBytes("roles")));
        rows.put(row.id, row);
      }
    }

    Map<String, RoleRow> byNewId = new LinkedHashMap<>();
    for (RoleRow row : rows.values()) {
      if (rename(row.id).equals(row.id)) {
        byNewId.put(row.id, row);
      }
    }
    List<RoleRow> merged = new ArrayList<>();
    for (RoleRow row : rows.values()) {
      String newId = rename(row.id);
      if (newId.equals(row.id)) {
        continue;
      }
      RoleRow target = byNewId.get(newId);
      if (target != null) {
        target.privileges.addAll(row.privileges);
        target.roles.addAll(row.roles);
        merged.add(row);
      }
      else {
        row.newId = newId;
        byNewId.put(newId, row);
      }
    }

    int updated = 0;
    for (RoleRow row : merged) {
      try (PreparedStatement delete = connection.prepareStatement("DELETE FROM role WHERE id = ?")) {
        delete.setString(1, row.id);
        delete.executeUpdate();
      }
    }
    for (RoleRow row : byNewId.values()) {
      List<String> privileges = renameAll(row.privileges);
      List<String> roles = renameAll(row.roles);
      roles.remove(row.newId);
      if (row.newId.equals(row.id) && privileges.equals(row.originalPrivileges) && roles.equals(row.originalRoles)) {
        continue;
      }
      try (PreparedStatement update = connection.prepareStatement(
          "UPDATE role SET id = ?, name = ?, privileges = ?, roles = ? WHERE id = ?")) {
        update.setString(1, row.newId);
        update.setString(2, row.id.equals(row.name) ? row.newId : row.name);
        setJsonParameter(update, 3, OBJECT_MAPPER.writeValueAsBytes(privileges), h2);
        setJsonParameter(update, 4, OBJECT_MAPPER.writeValueAsBytes(roles), h2);
        update.setString(5, row.id);
        update.executeUpdate();
      }
      updated++;
    }
    log.info("Rewrote built-in role and privilege references in {} roles, merged {} roles", updated, merged.size());
  }

  private void migrateUserRoleMappings(final Connection connection, final boolean h2) throws SQLException, IOException {
    List<MappingChange> changes = new ArrayList<>();
    try (PreparedStatement select = connection.prepareStatement("SELECT user_id, source, roles FROM user_role_mapping");
         ResultSet rs = select.executeQuery()) {
      while (rs.next()) {
        List<String> roles = readList(rs.getBytes("roles"));
        List<String> renamed = renameAll(roles);
        if (!renamed.equals(roles)) {
          changes.add(new MappingChange(rs.getString("user_id"), rs.getString("source"), renamed));
        }
      }
    }
    for (MappingChange change : changes) {
      try (PreparedStatement update = connection.prepareStatement(
          "UPDATE user_role_mapping SET roles = ? WHERE user_id = ? AND source = ?")) {
        setJsonParameter(update, 1, OBJECT_MAPPER.writeValueAsBytes(change.roles()), h2);
        update.setString(2, change.userId());
        update.setString(3, change.source());
        update.executeUpdate();
      }
    }
    log.info("Rewrote built-in role references in {} user role mappings", changes.size());
  }

  private void migrateDefaultRoleCapabilities(final Connection connection, final boolean h2)
      throws SQLException, IOException
  {
    Map<Object, Map<String, Object>> changes = new LinkedHashMap<>();
    try (PreparedStatement select = connection.prepareStatement(
        "SELECT id, properties FROM capability_storage_item WHERE type = ?")) {
      select.setString(1, DEFAULT_ROLE_CAPABILITY_TYPE);
      try (ResultSet rs = select.executeQuery()) {
        while (rs.next()) {
          byte[] json = rs.getBytes("properties");
          if (json == null) {
            continue;
          }
          Map<String, Object> properties = OBJECT_MAPPER.readValue(json, PROPERTIES_MAP);
          if (properties != null && properties.get(DEFAULT_ROLE_PROPERTY) instanceof String role) {
            String newRole = rename(role);
            if (!newRole.equals(role)) {
              properties.put(DEFAULT_ROLE_PROPERTY, newRole);
              changes.put(rs.getObject("id"), properties);
            }
          }
        }
      }
    }
    for (Map.Entry<Object, Map<String, Object>> change : changes.entrySet()) {
      try (PreparedStatement update = connection.prepareStatement(
          "UPDATE capability_storage_item SET properties = ? WHERE id = ?")) {
        setJsonParameter(update, 1, OBJECT_MAPPER.writeValueAsBytes(change.getValue()), h2);
        update.setObject(2, change.getKey());
        update.executeUpdate();
      }
    }
    log.info("Rewrote the role of {} default role capabilities", changes.size());
  }

  private static List<String> readList(final byte[] json) throws IOException {
    if (json == null) {
      return new ArrayList<>();
    }
    List<String> values = OBJECT_MAPPER.readValue(json, STRING_LIST);
    return values == null ? new ArrayList<>() : new ArrayList<>(values);
  }

  private static List<String> renameAll(final Collection<String> ids) {
    Set<String> result = new LinkedHashSet<>();
    for (String id : ids) {
      result.add(rename(id));
    }
    return new ArrayList<>(result);
  }

  private static final class RoleRow
  {
    private final String id;

    private final String name;

    private final List<String> originalPrivileges;

    private final List<String> originalRoles;

    private final Set<String> privileges;

    private final Set<String> roles;

    private String newId;

    private RoleRow(final String id, final String name, final List<String> privileges, final List<String> roles) {
      this.id = id;
      this.name = name;
      this.originalPrivileges = privileges;
      this.originalRoles = roles;
      this.privileges = new LinkedHashSet<>(privileges);
      this.roles = new LinkedHashSet<>(roles);
      this.newId = id;
    }
  }

  private record MappingChange(String userId, String source, List<String> roles)
  {
  }
}
