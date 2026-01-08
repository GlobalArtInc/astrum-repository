
package org.globalart.astrum.quartz.internal.upgrades;

import java.sql.Connection;
import java.util.Locale;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.quartz.internal.datastore.QuartzDAO;
import org.globalart.astrum.quartz.internal.datastore.QuartzJobDataTypeHandler;
import org.globalart.astrum.quartz.internal.datastore.QuartzTestDAO;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalToIgnoringCase;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

/**
 * Unit tests for {@link JobDescriptionMigrationStep_1_3} class
 */
public class JobDescriptionMigrationStep_1_3Test
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule()
      .handle(new QuartzJobDataTypeHandler())
      .access(QuartzDAO.class)
      .access(QuartzTestDAO.class);

  private DataSession<?> session;

  private QuartzTestDAO dao;

  private JobDescriptionMigrationStep_1_3 upgradeStep;

  private final static String CHARVAR = "character varying";

  private final static String TEXT = "text";

  private final static String ONE_BILLION_CHARACTERS = "1000000000";

  @Before
  public void setup() {
    upgradeStep = new JobDescriptionMigrationStep_1_3();
    session = sessionRule.openSession(DEFAULT_DATASTORE_NAME);
    dao = session.access(QuartzTestDAO.class);
  }

  @After
  public void cleanup() {
    session.close();
  }

  @Test
  public void testMigration() throws Exception {
    try (Connection conn = sessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      upgradeStep.migrate(conn);
    }

    String qrtzJobDetailsTableName = "qrtz_job_details";
    String quartzTriggersTableName = "qrtz_triggers";
    String columnName = "description";

    if ("H2".equals(session.sqlDialect())) {
      qrtzJobDetailsTableName = qrtzJobDetailsTableName.toUpperCase(Locale.ROOT);
      quartzTriggersTableName = quartzTriggersTableName.toUpperCase(Locale.ROOT);
      columnName = columnName.toUpperCase(Locale.ROOT);
      assertThat(dao.getTableColumnType(qrtzJobDetailsTableName, columnName), equalToIgnoringCase(CHARVAR));
      assertThat(dao.getTableColumnType(quartzTriggersTableName, columnName), equalToIgnoringCase(CHARVAR));

      // h2 version 2.2.224 no longer supports 'text' data_type, but the max length
      // of character varying is set high for this column, so we are now testing the length as well
      assertThat(dao.getColumnCharacterLimit(qrtzJobDetailsTableName, columnName), is(ONE_BILLION_CHARACTERS));
      assertThat(dao.getColumnCharacterLimit(quartzTriggersTableName, columnName), is(ONE_BILLION_CHARACTERS));
    }
    else {
      assertThat(dao.getTableColumnType(qrtzJobDetailsTableName, columnName), equalToIgnoringCase(TEXT));
      assertThat(dao.getTableColumnType(quartzTriggersTableName, columnName), equalToIgnoringCase(TEXT));
    }
  }
}
