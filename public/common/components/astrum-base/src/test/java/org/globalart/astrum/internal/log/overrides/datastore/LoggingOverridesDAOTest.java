
package org.globalart.astrum.internal.log.overrides.datastore;

import java.util.Optional;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.log.LoggerLevel;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

/**
 * {@link LoggingOverridesDAO} tests
 */
public class LoggingOverridesDAOTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(LoggingOverridesDAO.class);

  private DataSession<?> session;

  private LoggingOverridesDAO dao;

  private static final String FAKE_NAME = "funnyName";

  private static final String FAKE_LEVEL = LoggerLevel.DEBUG.toString();

  @Before
  public void setup() {
    session = sessionRule.openSession(DEFAULT_DATASTORE_NAME);
    dao = session.access(LoggingOverridesDAO.class);
  }

  @After
  public void cleanup() {
    session.close();
  }

  @Test
  public void testBrowse() {
    Continuation<LoggingOverridesData> emptyData = dao.readRecords(null);
    assertThat(emptyData.isEmpty(), is(true));

    LoggingOverridesData recordOne = new LoggingOverridesData();
    recordOne.setName(FAKE_NAME + "_ONE");
    recordOne.setLevel(FAKE_LEVEL);
    dao.createRecord(recordOne);

    LoggingOverridesData recordTwo = new LoggingOverridesData();
    recordTwo.setName(FAKE_NAME + "_TWO");
    recordTwo.setLevel(FAKE_LEVEL);
    dao.createRecord(recordTwo);

    LoggingOverridesData recordThree = new LoggingOverridesData();
    recordThree.setName(FAKE_NAME + "_THREE");
    recordThree.setLevel(FAKE_LEVEL);
    dao.createRecord(recordThree);

    Continuation<LoggingOverridesData> records = dao.readRecords(null);
    assertThat(records.size(), is(3));
  }

  @Test
  public void testCreateRecord() {
    LoggingOverridesData data = new LoggingOverridesData();
    data.setName(FAKE_NAME);
    data.setLevel(FAKE_LEVEL);

    dao.createRecord(data);
    Optional<LoggingOverridesData> record = dao.readRecords(null).stream().findFirst();
    assertThat(record.isPresent(), is(true));
    assertThat(record.get().getName(), is(FAKE_NAME));
    assertThat(record.get().getLevel(), is(FAKE_LEVEL));
  }

  @Test
  public void testUpdate() {
    LoggingOverridesData data = new LoggingOverridesData();
    data.setName(FAKE_NAME);
    data.setLevel(FAKE_LEVEL);
    dao.createRecord(data);

    Optional<LoggingOverridesData> initial = dao.readRecords(null).stream().findFirst();
    assertThat(initial.isPresent(), is(true));

    data.setLevel(LoggerLevel.INFO.toString());
    dao.updateRecord(data);

    Optional<LoggingOverridesData> updated = dao.readRecords(null).stream().findFirst();
    assertThat(updated.isPresent(), is(true));
    assertThat(updated.get().getLevel(), is(LoggerLevel.INFO.toString()));
  }

  @Test
  public void testDeleteByName() {
    LoggingOverridesData dataOne = new LoggingOverridesData();
    dataOne.setName(FAKE_NAME + "_ONE");
    dataOne.setLevel(FAKE_LEVEL);
    dao.createRecord(dataOne);

    LoggingOverridesData dataTwo = new LoggingOverridesData();
    dataTwo.setName(FAKE_NAME + "_TWO");
    dataTwo.setLevel(FAKE_LEVEL);
    dao.createRecord(dataTwo);

    Continuation<LoggingOverridesData> records = dao.readRecords(null);
    assertThat(records.size(), is(2));

    dao.deleteRecord(FAKE_NAME + "_ONE");
    records = dao.readRecords(null);
    assertThat(records.size(), is(1));

    dao.deleteRecord(FAKE_NAME + "_TWO");
    records = dao.readRecords(null);
    assertThat(records.size(), is(0));
  }
}
