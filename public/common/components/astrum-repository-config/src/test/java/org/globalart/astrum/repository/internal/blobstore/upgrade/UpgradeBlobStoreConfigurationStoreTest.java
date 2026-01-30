
package org.globalart.astrum.repository.internal.blobstore.upgrade;

import java.nio.charset.StandardCharsets;
import java.sql.Connection;
import java.sql.PreparedStatement;
import java.util.List;
import java.util.UUID;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.repository.internal.blobstore.BlobStoreConfigurationDAO;
import org.globalart.astrum.testdb.DataSessionConfiguration;
import org.globalart.astrum.testdb.DatabaseTest;
import org.globalart.astrum.testdb.TestDataSessionSupplier;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * Real-database tests for {@link UpgradeBlobStoreConfigurationStore} against {@code blob_store_configuration}.
 * These run against H2 only; PostgreSQL parity (the {@code JSON} vs {@code JSONB} {@code attributes} column
 * read path) is not exercised here and will be covered by the forthcoming {@code UpgradeMatrixIT} in
 * {@code astrum-integration-tests}.
 */
class UpgradeBlobStoreConfigurationStoreTest
{
  @DataSessionConfiguration(daos = {BlobStoreConfigurationDAO.class})
  TestDataSessionSupplier dataSessionSupplier;

  private UpgradeBlobStoreConfigurationStore store() {
    return new UpgradeBlobStoreConfigurationStore(dataSessionSupplier);
  }

  @DatabaseTest
  void list_empty_returnsEmpty() {
    assertThat(store().list()).isEmpty();
  }

  @DatabaseTest
  void list_returnsConfigurationsWithReconstructedAttributes() throws Exception {
    insert("s3-store", "S3", "{\"s3\":{\"bucket\":\"my-bucket\"}}");
    insert("file-store", "File", "{\"file\":{\"path\":\"default\"}}");

    List<BlobStoreConfiguration> list = store().list();

    assertThat(list).hasSize(2);
    BlobStoreConfiguration s3 = list.stream()
        .filter(c -> "s3-store".equals(c.getName()))
        .findFirst()
        .orElseThrow();
    assertThat(s3.getType()).isEqualTo("S3");
    assertThat(s3.attributes("s3").get("bucket")).isEqualTo("my-bucket");
  }

  private void insert(final String name, final String type, final String attributesJson) throws Exception {
    try (Connection conn = dataSessionSupplier.openConnection();
        PreparedStatement ps = conn.prepareStatement(
            "INSERT INTO blob_store_configuration (id, name, type, attributes) VALUES (?, ?, ?, ?)")) {
      boolean h2 = "H2".equals(conn.getMetaData().getDatabaseProductName());
      ps.setObject(1, UUID.randomUUID());
      ps.setString(2, name);
      ps.setString(3, type);
      byte[] json = attributesJson.getBytes(StandardCharsets.UTF_8);
      if (h2) {
        ps.setBytes(4, json);
      }
      else {
        ps.setString(4, attributesJson);
      }
      ps.executeUpdate();
    }
  }
}
