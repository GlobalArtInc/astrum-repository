
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.sql.PreparedStatement;
import java.util.Optional;
import org.springframework.beans.factory.annotation.Autowired;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.globalart.astrum.common.db.DatabaseCheck;

import static java.nio.charset.StandardCharsets.UTF_8;
import org.springframework.stereotype.Component;

/**
 * Convert REPLICATION_ONLY write policy to DENY.
 */
@Component
public class ConvertReplicationToDenyStep_1_37
    implements DatabaseMigrationStep
{
  private final DatabaseCheck databaseCheck;

  private final ObjectMapper mapper;

  private static final int PAGE_SIZE = 1000;

  private static final String UPDATE_ATTRIBUTES_BY_ID = "UPDATE repository SET attributes = ? WHERE id = ?;";

  @Autowired
  public ConvertReplicationToDenyStep_1_37(
      final DatabaseCheck databaseCheck)
  {
    this.mapper = new ObjectMapper();
    this.databaseCheck = databaseCheck;
  }

  @Override
  public Optional<String> version() {
    return Optional.of("1.37");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    int totalRows = getTotalRowCount(connection);
    int totalPages = (int) Math.ceil((double) totalRows / PAGE_SIZE);

    for (int i = 0; i < totalPages; i++) {
      int offset = i * PAGE_SIZE;
      String selectQuery = "SELECT id, attributes FROM repository LIMIT ? OFFSET ?";
      try (PreparedStatement ps = connection.prepareStatement(selectQuery)) {
        ps.setInt(1, PAGE_SIZE);
        ps.setInt(2, offset);
        try (ResultSet rs = ps.executeQuery()) {
          while (rs.next()) {
            String id = rs.getString("id");
            String attributes = rs.getString("attributes");

            ObjectNode attributesNode = (ObjectNode) mapper.readTree(attributes);
            JsonNode storageAttributes = attributesNode.get("storage");

            if (storageAttributes != null) {
              JsonNode writePolicyNode = storageAttributes.get("writePolicy");
              if (writePolicyNode != null && "REPLICATION_ONLY".equals(writePolicyNode.asText())) {
                ((ObjectNode) storageAttributes).put("writePolicy", "DENY");
                attributesNode.set("storage", storageAttributes);
                updateAttributes(connection, id, mapper.writeValueAsBytes(attributesNode));
              }
            }
          }
        }
      }
      catch (SQLException | JsonProcessingException e) {
        throw new RuntimeException(e);
      }
    }
  }

  private int getTotalRowCount(Connection connection) throws SQLException {
    String countQuery = "SELECT COUNT(id) FROM repository";
    try (PreparedStatement ps = connection.prepareStatement(countQuery);
        ResultSet rs = ps.executeQuery()) {
      if (rs.next()) {
        return rs.getInt(1);
      }
      else {
        throw new SQLException("Failed to count rows in repository table");
      }
    }
  }

  private void updateAttributes(Connection connection, String id, byte[] attributes) throws SQLException {
    try (PreparedStatement ps = connection.prepareStatement(UPDATE_ATTRIBUTES_BY_ID)) {
      if (!databaseCheck.isPostgresql()) {
        ps.setBytes(1, attributes);
      }
      else {
        ps.setString(1, new String(attributes, UTF_8));
      }
      ps.setString(2, id);
      ps.executeUpdate();
    }
  }
}
