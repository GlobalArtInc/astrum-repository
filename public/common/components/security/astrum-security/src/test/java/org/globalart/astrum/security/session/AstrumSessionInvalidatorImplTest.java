
package org.globalart.astrum.security.session;

import java.io.Serializable;
import java.util.Arrays;
import java.util.Collections;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditRecorder;

import org.apache.shiro.cache.Cache;
import org.apache.shiro.astrum.AstrumWebSessionManager;
import org.apache.shiro.session.Session;
import org.apache.shiro.session.mgt.eis.CachingSessionDAO;
import org.apache.shiro.session.mgt.eis.SessionDAO;
import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.subject.SimplePrincipalCollection;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class AstrumSessionInvalidatorImplTest
{
  private static final String PRINCIPALS_SESSION_KEY =
      "org.apache.shiro.subject.support.DefaultSubjectContext_PRINCIPALS_SESSION_KEY";

  @Mock
  private AstrumWebSessionManager sessionManager;

  @Mock
  private AuditRecorder auditRecorder;

  @Mock
  private CachingSessionDAO cachingSessionDAO;

  @Mock
  private Cache<Serializable, Session> cache;

  private AstrumSessionInvalidatorImpl underTest;

  @Before
  public void setup() {
    underTest = new AstrumSessionInvalidatorImpl(sessionManager, auditRecorder);
  }

  @Test
  public void invalidate_deletesOnlyMatchingUserSessions() {
    Session aliceSession = sessionWithPrincipal("alice");
    Session bobSession = sessionWithPrincipal("bob");
    Session aliceSession2 = sessionWithPrincipal("alice");

    when(sessionManager.getSessionDAO()).thenReturn(cachingSessionDAO);
    when(cachingSessionDAO.getActiveSessionsCache()).thenReturn(cache);
    when(cache.values()).thenReturn(Arrays.asList(aliceSession, bobSession, aliceSession2));

    int count = underTest.invalidateSessionsForUser("alice", "default", "password change");

    assertThat(count, is(2));
    verify(cachingSessionDAO).delete(aliceSession);
    verify(cachingSessionDAO).delete(aliceSession2);
    verify(cachingSessionDAO, never()).delete(bobSession);
  }

  @Test
  public void invalidate_sessionDaoNotCaching_returnsZero() {
    SessionDAO nonCachingDAO = mock(SessionDAO.class);
    when(sessionManager.getSessionDAO()).thenReturn(nonCachingDAO);

    int count = underTest.invalidateSessionsForUser("alice", "default", "password change");

    assertThat(count, is(0));
    verify(nonCachingDAO, never()).delete(any(Session.class));
  }

  @Test
  public void invalidate_nullCache_returnsZero() {
    when(sessionManager.getSessionDAO()).thenReturn(cachingSessionDAO);
    when(cachingSessionDAO.getActiveSessionsCache()).thenReturn(null);

    int count = underTest.invalidateSessionsForUser("alice", "default", "password change");

    assertThat(count, is(0));
    verify(cachingSessionDAO, never()).delete(any(Session.class));
  }

  @Test
  public void invalidate_auditRecorded_whenCountPositive() {
    Session aliceSession = sessionWithPrincipal("alice");
    when(sessionManager.getSessionDAO()).thenReturn(cachingSessionDAO);
    when(cachingSessionDAO.getActiveSessionsCache()).thenReturn(cache);
    when(cache.values()).thenReturn(Collections.singletonList(aliceSession));
    when(auditRecorder.isEnabled()).thenReturn(true);

    underTest.invalidateSessionsForUser("alice", "default", "password change");

    ArgumentCaptor<AuditData> captor = ArgumentCaptor.forClass(AuditData.class);
    verify(auditRecorder).record(captor.capture());
    AuditData data = captor.getValue();
    assertThat(data.getDomain(), is("security.session"));
    assertThat(data.getType(), is("user-session-invalidation"));
    assertThat(data.getAttributes().get("username"), is("alice"));
    assertThat(data.getAttributes().get("sessionType"), is("session"));
    assertThat(data.getAttributes().get("sessionCount"), is("1"));
    assertThat(data.getAttributes().get("reason"), is("password change"));
  }

  @Test
  public void invalidate_auditNotRecorded_whenNoSessionsDeleted() {
    Session bobSession = sessionWithPrincipal("bob");
    when(sessionManager.getSessionDAO()).thenReturn(cachingSessionDAO);
    when(cachingSessionDAO.getActiveSessionsCache()).thenReturn(cache);
    when(cache.values()).thenReturn(Collections.singletonList(bobSession));
    when(auditRecorder.isEnabled()).thenReturn(true);

    underTest.invalidateSessionsForUser("alice", "default", "password change");

    verify(auditRecorder, never()).record(any(AuditData.class));
  }

  private static Session sessionWithPrincipal(final String username) {
    Session session = mock(Session.class);
    PrincipalCollection principals = new SimplePrincipalCollection(username, "anyRealm");
    when(session.getAttribute(PRINCIPALS_SESSION_KEY)).thenReturn(principals);
    return session;
  }
}
