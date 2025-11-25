
package org.globalart.astrum.testdb.example;

import java.sql.Connection;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.SchemaTemplate;
import org.globalart.astrum.testdb.DataSessionRule;
import org.globalart.astrum.testdb.example.template.metal.MetalSpindleDAO;
import org.globalart.astrum.testdb.example.template.metal.MetalSprocketDAO;
import org.globalart.astrum.testdb.example.template.plastic.PlasticSpindleDAO;
import org.globalart.astrum.testdb.example.template.plastic.PlasticSprocketDAO;

import org.hamcrest.Matcher;
import org.junit.Rule;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.globalart.astrum.common.text.Strings2.lower;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

/**
 * Test the {@link SchemaTemplate} annotation.
 */
public class SchemaTemplateTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule()
      .access(PlasticSpindleDAO.class)
      .access(PlasticSprocketDAO.class)
      .access(MetalSpindleDAO.class)
      .access(MetalSprocketDAO.class);

  @Test
  public void testExpectedAccessTypesRegisteredFirst() throws SQLException {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      // will fail if @Expects and @SchemaTemplate are not respected
      session.access(MetalSprocketDAO.class);
    }

    // check the extra column added by plastic_sprocket exists
    assertColumns("metal_sprocket", containsInAnyOrder("id", "spindle_id", "widget_id"));
    assertColumns("plastic_sprocket", containsInAnyOrder("id", "spindle_id", "widget_id", "notes"));
  }

  private void assertColumns(final String tableName, final Matcher<Iterable<?>> matcher) throws SQLException {
    List<String> columnNames = new ArrayList<>();
    try (Connection connection = sessionRule.openConnection(DEFAULT_DATASTORE_NAME);
        ResultSet rs = connection.getMetaData().getColumns(null, null, "%", null)) {

      while (rs.next()) {
        if (tableName.equalsIgnoreCase(rs.getString("TABLE_NAME"))) {
          columnNames.add(lower(rs.getString("COLUMN_NAME")));
        }
      }
    }
    assertThat(columnNames, matcher);
  }
}
