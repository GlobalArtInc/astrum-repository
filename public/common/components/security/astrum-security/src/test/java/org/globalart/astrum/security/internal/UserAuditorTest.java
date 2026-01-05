
package org.globalart.astrum.security.internal;

import java.util.Set;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditRecorder;
import org.globalart.astrum.security.role.RoleIdentifier;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserCreatedEvent;
import org.globalart.astrum.security.user.UserDeletedEvent;
import org.globalart.astrum.security.user.UserStatus;
import org.globalart.astrum.security.user.UserUpdatedEvent;

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
public class UserAuditorTest
{
  @Mock
  private AuditRecorder auditRecorder;

  @Captor
  private ArgumentCaptor<AuditData> auditDataCaptor;

  private UserAuditor auditor;

  private User testUser;

  @Before
  public void setup() {
    auditor = new UserAuditor();
    auditor.setAuditRecorder(auditRecorder);

    testUser = new User();
    testUser.setUserId("testuser");
    testUser.setFirstName("Test");
    testUser.setLastName("User");
    testUser.setEmailAddress("test@example.com");
    testUser.setSource("default");
    testUser.setStatus(UserStatus.active);
    testUser.setRoles(Set.of(new RoleIdentifier("default", "as-admin")));
  }

  @Test
  public void testUserCreatedEvent_whenRecording() {
    when(auditRecorder.isEnabled()).thenReturn(true);

    auditor.on(new UserCreatedEvent(testUser));

    verify(auditRecorder).record(auditDataCaptor.capture());
    AuditData data = auditDataCaptor.getValue();

    assertThat(data.getDomain(), is("security.user"));
    assertThat(data.getType(), is("created"));
    assertThat(data.getContext(), is("testuser"));
    assertThat(data.getAttributes().get("id"), is("testuser"));
    assertThat(data.getAttributes().get("name"), is("Test User"));
    assertThat(data.getAttributes().get("email"), is("test@example.com"));
    assertThat(data.getAttributes().get("source"), is("default"));
    assertThat(data.getAttributes().get("status"), is("active"));
    assertThat(data.getAttributes().get("roles"), is("as-admin"));
  }

  @Test
  public void testUserUpdatedEvent_whenRecording() {
    when(auditRecorder.isEnabled()).thenReturn(true);

    auditor.on(new UserUpdatedEvent(testUser));

    verify(auditRecorder).record(auditDataCaptor.capture());
    AuditData data = auditDataCaptor.getValue();

    assertThat(data.getDomain(), is("security.user"));
    assertThat(data.getType(), is("updated"));
    assertThat(data.getContext(), is("testuser"));
  }

  @Test
  public void testUserDeletedEvent_whenRecording() {
    when(auditRecorder.isEnabled()).thenReturn(true);

    auditor.on(new UserDeletedEvent(testUser));

    verify(auditRecorder).record(auditDataCaptor.capture());
    AuditData data = auditDataCaptor.getValue();

    assertThat(data.getDomain(), is("security.user"));
    assertThat(data.getType(), is("deleted"));
    assertThat(data.getContext(), is("testuser"));
  }

  @Test
  public void testNoAudit_whenNotRecording() {
    when(auditRecorder.isEnabled()).thenReturn(false);

    auditor.on(new UserCreatedEvent(testUser));

    verify(auditRecorder, never()).record(any());
  }
}
