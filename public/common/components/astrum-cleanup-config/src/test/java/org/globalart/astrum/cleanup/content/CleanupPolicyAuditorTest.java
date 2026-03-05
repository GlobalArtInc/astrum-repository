
package org.globalart.astrum.cleanup.content;

import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditRecorder;
import org.globalart.astrum.cleanup.internal.storage.CleanupPolicyData;
import org.globalart.astrum.common.event.EventHelper;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class CleanupPolicyAuditorTest
{
  private static final String POLICY_NAME = "test-cleanup-policy";

  private static final String POLICY_FORMAT = "maven2";

  private static final String POLICY_NOTES = "test description";

  @Mock
  private AuditRecorder auditRecorder;

  private TestableCleanupPolicyAuditor underTest;

  private CleanupPolicyData cleanupPolicy;

  private Map<String, String> criteria;

  @Before
  public void setUp() {
    underTest = new TestableCleanupPolicyAuditor();
    underTest.setAuditRecorder(auditRecorder);
    when(auditRecorder.isEnabled()).thenReturn(true);

    criteria = new HashMap<>();
    criteria.put("lastBlobUpdated", "30");

    cleanupPolicy = new CleanupPolicyData();
    cleanupPolicy.setName(POLICY_NAME);
    cleanupPolicy.setFormat(POLICY_FORMAT);
    cleanupPolicy.setNotes(POLICY_NOTES);
    cleanupPolicy.setCriteria(criteria);
  }

  @Test
  public void testDomainConstant() {
    assertThat(CleanupPolicyAuditor.DOMAIN, is("cleanupPolicy"));
  }

  @Test
  public void testConstruction_registersEventTypes() {
    assertThat(underTest.resolveType(CleanupPolicyCreatedEvent.class), is("created"));
    assertThat(underTest.resolveType(CleanupPolicyUpdatedEvent.class), is("updated"));
    assertThat(underTest.resolveType(CleanupPolicyDeletedEvent.class), is("deleted"));
  }

  @Test
  public void testOnCreatedEvent_recording() {
    CleanupPolicyCreatedEvent event = new CleanupPolicyCreatedEvent(cleanupPolicy);

    underTest.on(event);

    AuditData auditData = captureRecordedAuditData();
    assertThat(auditData.getDomain(), is(CleanupPolicyAuditor.DOMAIN));
    assertThat(auditData.getType(), is("created"));
    assertThat(auditData.getContext(), is(POLICY_NAME));

    Map<String, Object> attributes = auditData.getAttributes();
    assertThat(attributes.keySet(), containsInAnyOrder("format", "description", "criteria"));
    assertThat(attributes.get("format"), is(POLICY_FORMAT));
    assertThat(attributes.get("description"), is(POLICY_NOTES));
    assertThat(attributes.get("criteria"), is(criteria));
  }

  @Test
  public void testOnUpdatedEvent_recording() {
    CleanupPolicyUpdatedEvent event = new CleanupPolicyUpdatedEvent(cleanupPolicy);

    underTest.on(event);

    AuditData auditData = captureRecordedAuditData();
    assertThat(auditData.getDomain(), is(CleanupPolicyAuditor.DOMAIN));
    assertThat(auditData.getType(), is("updated"));
    assertThat(auditData.getContext(), is(POLICY_NAME));

    Map<String, Object> attributes = auditData.getAttributes();
    assertThat(attributes.keySet(), containsInAnyOrder("format", "description", "criteria"));
    assertThat(attributes.get("format"), is(POLICY_FORMAT));
    assertThat(attributes.get("description"), is(POLICY_NOTES));
    assertThat(attributes.get("criteria"), is(criteria));
  }

  @Test
  public void testOnDeletedEvent_recording() {
    CleanupPolicyDeletedEvent event = new CleanupPolicyDeletedEvent(cleanupPolicy);

    underTest.on(event);

    AuditData auditData = captureRecordedAuditData();
    assertThat(auditData.getDomain(), is(CleanupPolicyAuditor.DOMAIN));
    assertThat(auditData.getType(), is("deleted"));
    assertThat(auditData.getContext(), is(POLICY_NAME));

    Map<String, Object> attributes = auditData.getAttributes();
    assertThat(attributes.keySet(), containsInAnyOrder("format", "description", "criteria"));
    assertThat(attributes.get("format"), is(POLICY_FORMAT));
    assertThat(attributes.get("description"), is(POLICY_NOTES));
    assertThat(attributes.get("criteria"), is(criteria));
  }

  @Test
  public void testOnEvent_recordsNullPolicyAttributes() {
    CleanupPolicyData emptyPolicy = new CleanupPolicyData();
    emptyPolicy.setName(POLICY_NAME);

    underTest.on(new CleanupPolicyCreatedEvent(emptyPolicy));

    AuditData auditData = captureRecordedAuditData();
    assertThat(auditData.getDomain(), is(CleanupPolicyAuditor.DOMAIN));
    assertThat(auditData.getType(), is("created"));
    assertThat(auditData.getContext(), is(POLICY_NAME));

    // the auditor unconditionally writes all three attribute keys even when the policy values are null
    Map<String, Object> attributes = auditData.getAttributes();
    assertThat(attributes.keySet(), containsInAnyOrder("format", "description", "criteria"));
    assertThat(attributes.get("format"), is(nullValue()));
    assertThat(attributes.get("description"), is(nullValue()));
    assertThat(attributes.get("criteria"), is(nullValue()));
  }

  @Test
  public void testOnEvent_notRecording() {
    when(auditRecorder.isEnabled()).thenReturn(false);

    CleanupPolicyCreatedEvent event = new CleanupPolicyCreatedEvent(cleanupPolicy);

    underTest.on(event);

    verify(auditRecorder, never()).record(any());
  }

  @Test
  public void testOnEvent_notRecordingWhenReplicating() {
    // recorder is enabled (see setUp) but replication is in progress, so isRecording() must be false
    CleanupPolicyCreatedEvent event = new CleanupPolicyCreatedEvent(cleanupPolicy);

    EventHelper.asReplicating(() -> underTest.on(event));

    verify(auditRecorder, never()).record(any());
  }

  private AuditData captureRecordedAuditData() {
    ArgumentCaptor<AuditData> captor = ArgumentCaptor.forClass(AuditData.class);
    verify(auditRecorder).record(captor.capture());
    return captor.getValue();
  }

  /**
   * Exposes the protected {@link org.globalart.astrum.audit.AuditorSupport#type(Class)} lookup so the registrations
   * performed by the {@link CleanupPolicyAuditor} constructor can be asserted from this package.
   */
  private static class TestableCleanupPolicyAuditor
      extends CleanupPolicyAuditor
  {
    String resolveType(final Class<?> type) {
      return type(type);
    }
  }
}
