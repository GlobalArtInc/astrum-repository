
package org.globalart.astrum.upgrade.datastore.internal.audit;

import java.util.Collection;
import java.util.Collections;
import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditRecorder;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.common.upgrade.events.UpgradeCompletedEvent;
import org.globalart.astrum.common.upgrade.events.UpgradeFailedEvent;
import org.globalart.astrum.common.upgrade.events.UpgradeStartedEvent;

import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.mockito.Mock;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.upgrade.datastore.internal.audit.UpgradeAuditor.COMPLETED;
import static org.globalart.astrum.upgrade.datastore.internal.audit.UpgradeAuditor.DOMAIN;
import static org.globalart.astrum.upgrade.datastore.internal.audit.UpgradeAuditor.FAILED;
import static org.globalart.astrum.upgrade.datastore.internal.audit.UpgradeAuditor.MESSAGE;
import static org.globalart.astrum.upgrade.datastore.internal.audit.UpgradeAuditor.MIGRATIONS;
import static org.globalart.astrum.upgrade.datastore.internal.audit.UpgradeAuditor.ASTRUM_VERSION;
import static org.globalart.astrum.upgrade.datastore.internal.audit.UpgradeAuditor.NODE_IDS;
import static org.globalart.astrum.upgrade.datastore.internal.audit.UpgradeAuditor.SCHEMA_VERSION;
import static org.globalart.astrum.upgrade.datastore.internal.audit.UpgradeAuditor.STARTED;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class UpgradeAuditorTest
{
  private static final String V3_99 = "3.99";

  private static final String ERROR_MSG = "Something unfortunate occurred";

  private static final String FOO_1_38 = "Foo_1.38";

  private static final String V1_38 = "1.38";

  private static final String V1_37 = "1.37";

  private static final String USER = "timmy";

  @Mock
  private AuditRecorder recorder;

  @Mock
  private ApplicationVersion applicationVersion;

  @Captor
  private ArgumentCaptor<AuditData> auditDataCaptor;

  private UpgradeAuditor underTest;

  @Before
  public void setup() {
    when(applicationVersion.getVersion()).thenReturn(V3_99);
    underTest = new UpgradeAuditor(applicationVersion);
    underTest.setAuditRecorder(recorder);

    when(recorder.isEnabled()).thenReturn(true);
  }

  @Test
  public void testOn_UpgradeCompletedEvent() {
    Collection<String> nodeIds = Collections.singletonList("my-node-id");
    underTest.on(new UpgradeCompletedEvent(USER, V1_38, nodeIds, FOO_1_38));

    verify(recorder).record(auditDataCaptor.capture());

    AuditData recorded = auditDataCaptor.getValue();
    assertAuditDataWithNodes(recorded, COMPLETED, USER, V1_38, V3_99, FOO_1_38, nodeIds);
  }

  @Test
  public void testOn_UpgradeFailedEvent() {
    underTest.on(new UpgradeFailedEvent(USER, V1_38, ERROR_MSG));

    verify(recorder).record(auditDataCaptor.capture());

    AuditData recorded = auditDataCaptor.getValue();
    assertAuditDataWithError(recorded, FAILED, USER, V1_38, V3_99, ERROR_MSG);
  }

  @Test
  public void testOn_UpgradeStartedEvent() {
    underTest.on(new UpgradeStartedEvent(USER, V1_37, FOO_1_38));

    verify(recorder).record(auditDataCaptor.capture());

    AuditData recorded = auditDataCaptor.getValue();
    assertAuditDataWithMigration(recorded, STARTED, USER, V1_37, V3_99, FOO_1_38);
  }

  /*
   * The schema version can be null when migration during the first Astrum start
   */
  @Test
  public void testOn_UpgradeStartedEvent_nullSchema() {
    underTest.on(new UpgradeStartedEvent(USER, null, FOO_1_38));

    verify(recorder).record(auditDataCaptor.capture());

    AuditData recorded = auditDataCaptor.getValue();
    assertAuditDataWithMigration(recorded, STARTED, USER, null, V3_99, FOO_1_38);
  }

  /*
   * The user can be null when an upgrade occurs during startup
   */
  @Test
  public void testOn_nullUser() {
    underTest.on(new UpgradeStartedEvent(null, V1_37, FOO_1_38));

    verify(recorder).record(auditDataCaptor.capture());

    AuditData recorded = auditDataCaptor.getValue();
    assertAuditDataWithMigration(recorded, STARTED, "system", V1_37, V3_99, FOO_1_38);
  }

  private static void assertAuditData(
      final AuditData actual,
      final String type,
      final String user,
      final String schemaVersion,
      final String astrumVersion)
  {
    assertThat(actual.getContext(), is("system"));
    assertThat(actual.getDomain(), is(DOMAIN));
    assertThat(actual.getInitiator(), is(user));
    assertThat(actual.getType(), is(type));

    Map<String, Object> actualAttributes = actual.getAttributes();
    assertThat(actualAttributes.get(SCHEMA_VERSION), is(schemaVersion));
    assertThat(actualAttributes.get(ASTRUM_VERSION), is(astrumVersion));
  }

  private static void assertAuditDataWithMigration(
      final AuditData actual,
      final String type,
      final String user,
      final String schemaVersion,
      final String astrumVersion,
      final String migrations)
  {
    assertAuditData(actual, type, user, schemaVersion, astrumVersion);
    assertThat(((String[]) (actual.getAttributes().get(MIGRATIONS)))[0], is(migrations));
  }

  private static void assertAuditDataWithNodes(
      final AuditData actual,
      final String type,
      final String user,
      final String schemaVersion,
      final String astrumVersion,
      final String migrations,
      final Collection<String> nodeIds)
  {
    assertAuditDataWithMigration(actual, type, user, schemaVersion, astrumVersion, migrations);
    assertThat(actual.getAttributes().get(NODE_IDS), is(nodeIds));
  }

  private static void assertAuditDataWithError(
      final AuditData actual,
      final String type,
      final String user,
      final String schemaVersion,
      final String astrumVersion,
      final String error)
  {
    assertAuditData(actual, type, user, schemaVersion, astrumVersion);
    assertThat(actual.getAttributes().get(MESSAGE), is(error));
  }
}
