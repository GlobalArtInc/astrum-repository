
package org.globalart.astrum.security.internal;

import java.util.Set;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditRecorder;
import org.globalart.astrum.security.role.Role;
import org.globalart.astrum.security.role.RoleCreatedEvent;
import org.globalart.astrum.security.role.RoleDeletedEvent;
import org.globalart.astrum.security.role.RoleUpdatedEvent;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class RoleAuditorTest
{
  @Mock
  private AuditRecorder auditRecorder;

  @Captor
  private ArgumentCaptor<AuditData> auditDataCaptor;

  private RoleAuditor auditor;

  private Role testRole;

  @Before
  public void setup() {
    auditor = new RoleAuditor();
    auditor.setAuditRecorder(auditRecorder);

    testRole = new Role();
    testRole.setRoleId("test-role");
    testRole.setName("Test Role");
    testRole.setDescription("A test role");
    testRole.setSource("default");
    testRole.setPrivileges(Set.of("as-all"));
    testRole.setRoles(Set.of("as-admin"));
  }

  @Test
  public void testRoleCreatedEvent_whenRecording() {
    when(auditRecorder.isEnabled()).thenReturn(true);

    auditor.on(new RoleCreatedEvent(testRole));

    verify(auditRecorder).record(auditDataCaptor.capture());
    AuditData data = auditDataCaptor.getValue();

    assertThat(data.getDomain(), is("security.role"));
    assertThat(data.getType(), is("created"));
    assertThat(data.getContext(), is("test-role"));
    assertThat(data.getAttributes().get("id"), is("test-role"));
    assertThat(data.getAttributes().get("name"), is("Test Role"));
    assertThat(data.getAttributes().get("source"), is("default"));
    assertThat(data.getAttributes().get("privileges"), is("as-all"));
    assertThat(data.getAttributes().get("roles"), is("as-admin"));
  }

  @Test
  public void testRoleUpdatedEvent_whenRecording() {
    when(auditRecorder.isEnabled()).thenReturn(true);

    auditor.on(new RoleUpdatedEvent(testRole));

    verify(auditRecorder).record(auditDataCaptor.capture());
    AuditData data = auditDataCaptor.getValue();

    assertThat(data.getDomain(), is("security.role"));
    assertThat(data.getType(), is("updated"));
    assertThat(data.getContext(), is("test-role"));
  }

  @Test
  public void testRoleDeletedEvent_whenRecording() {
    when(auditRecorder.isEnabled()).thenReturn(true);

    auditor.on(new RoleDeletedEvent(testRole));

    verify(auditRecorder).record(auditDataCaptor.capture());
    AuditData data = auditDataCaptor.getValue();

    assertThat(data.getDomain(), is("security.role"));
    assertThat(data.getType(), is("deleted"));
    assertThat(data.getContext(), is("test-role"));
  }

  @Test
  public void testNoAudit_whenNotRecording() {
    when(auditRecorder.isEnabled()).thenReturn(false);

    auditor.on(new RoleCreatedEvent(testRole));

    verify(auditRecorder, never()).record(any());
  }
}
