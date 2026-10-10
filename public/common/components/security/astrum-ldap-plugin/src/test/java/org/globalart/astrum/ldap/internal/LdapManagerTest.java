package org.globalart.astrum.ldap.internal;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicLong;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.kv.KeyValueEvent;
import org.globalart.astrum.security.authz.AuthorizationConfigurationChanged;
import org.globalart.astrum.ssl.TrustStore;

import org.apache.shiro.authc.AuthenticationException;
import org.apache.shiro.authc.IncorrectCredentialsException;
import org.apache.shiro.authc.UnknownAccountException;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.instanceOf;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.argThat;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

public class LdapManagerTest
{
  private LdapTestDirectory ldap;

  private final List<LdapServer> servers = new ArrayList<>();

  private LdapDirectory directory;

  private EventManager eventManager;

  private final AtomicLong now = new AtomicLong(1_000_000L);

  private LdapManager underTest;

  @Before
  public void setUp() throws Exception {
    ldap = new LdapTestDirectory();
    LdapConfigurationStore store = mock(LdapConfigurationStore.class);
    when(store.list()).thenAnswer(invocation -> servers.stream().map(LdapServer::copy).toList());
    when(store.isConfigured()).thenAnswer(invocation -> !servers.isEmpty());
    when(store.password(any())).thenAnswer(invocation -> LdapTestDirectory.systemPassword());
    directory = spy(new LdapDirectory(() -> mock(TrustStore.class)));
    eventManager = mock(EventManager.class);
    underTest = new LdapManager(store, directory, eventManager, new Time(5, TimeUnit.MINUTES));
    underTest.setClock(now::get);
  }

  @After
  public void tearDown() {
    ldap.close();
  }

  @Test
  public void authenticateReturnsTheUserWithGroups() {
    servers.add(ldap.dynamicServer("main"));
    LdapUser user = underTest.authenticate("Alice", "alice-secret".toCharArray());
    assertThat(user.username(), is("alice"));
    assertThat(user.groups(), containsInAnyOrder("developers", "admins"));
  }

  @Test
  public void wrongPasswordAndUnknownUser() {
    servers.add(ldap.dynamicServer("main"));
    assertThrows(IncorrectCredentialsException.class, () -> underTest.authenticate("alice", "nope".toCharArray()));
    assertThrows(UnknownAccountException.class, () -> underTest.authenticate("nobody", "nope".toCharArray()));
  }

  @Test
  public void nextServerIsTriedWhenOneIsDown() {
    servers.add(unreachable("down"));
    servers.add(ldap.dynamicServer("main"));
    assertThat(underTest.authenticate("alice", "alice-secret".toCharArray()).username(), is("alice"));
  }

  @Test
  public void unknownUserWithAServerDownIsNotReportedAsUnknown() {
    servers.add(ldap.dynamicServer("main"));
    servers.add(unreachable("down"));
    AuthenticationException e = assertThrows(AuthenticationException.class,
        () -> underTest.authenticate("nobody", "x".toCharArray()));
    assertThat(e instanceof UnknownAccountException, is(false));
  }

  @Test
  public void failingServerIsSkippedForTheRetryDelay() {
    LdapServer down = unreachable("down");
    down.setMaxIncidentsCount(2);
    down.setConnectionRetryDelaySeconds(60);
    servers.add(down);
    servers.add(ldap.dynamicServer("main"));

    underTest.authenticate("alice", "alice-secret".toCharArray());
    underTest.getUser("bob");
    underTest.getUser("carol");
    verify(directory, times(1)).authenticate(named("down"), any(), any(), any());
    verify(directory, times(1)).findUser(named("down"), any(), eq("bob"));
    verify(directory, never()).findUser(named("down"), any(), eq("carol"));

    now.addAndGet(61_000);
    // dave is out of reach of the main server, and the failing one is asked again
    assertThrows(LdapServerException.class, () -> underTest.getUser("dave"));
    verify(directory, times(1)).findUser(named("down"), any(), eq("dave"));
  }

  @Test
  public void successfulLoginIsCachedAsAHash() {
    servers.add(ldap.dynamicServer("main"));
    underTest.authenticate("alice", "alice-secret".toCharArray());
    ldap.stop();

    assertThat(underTest.authenticate("ALICE", "alice-secret".toCharArray()).username(), is("alice"));
    AuthenticationException e = assertThrows(AuthenticationException.class,
        () -> underTest.authenticate("alice", "other".toCharArray()));
    assertThat(e instanceof IncorrectCredentialsException, is(false));
  }

  @Test
  public void clearCacheForgetsLoginsAndRecomputesPermissions() {
    servers.add(ldap.dynamicServer("main"));
    underTest.authenticate("alice", "alice-secret".toCharArray());
    underTest.clearCache();
    ldap.stop();

    assertThrows(AuthenticationException.class, () -> underTest.authenticate("alice", "alice-secret".toCharArray()));
    verify(eventManager).post(any(AuthorizationConfigurationChanged.class));
  }

  @Test
  public void configurationChangeClearsTheCache() {
    servers.add(ldap.dynamicServer("main"));
    underTest.getUser("alice");
    underTest.on(new KeyValueEvent("unrelated", null));
    verify(eventManager, never()).post(any());
    underTest.on(new KeyValueEvent(LdapConfigurationStore.KEY, null));
    verify(eventManager).post(any(AuthorizationConfigurationChanged.class));
  }

  @Test
  public void getUserFailsWhenTheOnlyServerIsDown() {
    servers.add(unreachable("down"));
    assertThrows(LdapServerException.class, () -> underTest.getUser("alice"));
  }

  @Test
  public void getUserIsCached() {
    servers.add(ldap.dynamicServer("main"));
    assertThat(underTest.getUser("alice").isPresent(), is(true));
    assertThat(underTest.getUser("nobody").isPresent(), is(false));
    ldap.stop();
    assertThat(underTest.getUser("alice").isPresent(), is(true));
    assertThat(underTest.getUser("nobody").isPresent(), is(false));
  }

  @Test
  public void searchAcrossServersSkipsFailures() {
    servers.add(unreachable("down"));
    servers.add(ldap.staticServer("main"));
    servers.add(ldap.dynamicServer("copy"));
    assertThat(underTest.searchUsers("a", 100).stream().map(LdapUser::username).toList(), contains("alice"));
    assertThat(underTest.searchGroups("dev", 100), contains("developers"));
    assertThat(underTest.groupExists("admins"), is(true));
  }

  @Test
  public void verifyLoginReportsTheProblem() {
    LdapServer server = ldap.dynamicServer("main");
    assertThat(underTest.verifyLogin(server, LdapTestDirectory.systemPassword(), "alice",
        "alice-secret".toCharArray()).username(), is("alice"));
    LdapServerException wrong = assertThrows(LdapServerException.class,
        () -> underTest.verifyLogin(server, LdapTestDirectory.systemPassword(), "alice", "x".toCharArray()));
    assertThat(wrong.getMessage(), is("The password for user 'alice' is incorrect"));
    assertThat(assertThrows(Exception.class,
            () -> underTest.verifyLogin(server, LdapTestDirectory.systemPassword(), "nobody", "x".toCharArray())),
        instanceOf(LdapServerException.class));
  }

  private static LdapServer named(final String name) {
    return argThat(server -> server != null && name.equals(server.getName()));
  }

  private LdapServer unreachable(final String name) {
    LdapServer server = ldap.dynamicServer(name);
    server.setPort(1);
    server.setConnectionTimeoutSeconds(2);
    return server;
  }
}
