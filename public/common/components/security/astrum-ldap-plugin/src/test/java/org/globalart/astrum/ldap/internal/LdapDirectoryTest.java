package org.globalart.astrum.ldap.internal;

import java.util.List;
import java.util.Optional;
import java.util.Set;

import org.globalart.astrum.ldap.internal.LdapDirectory.Authentication;
import org.globalart.astrum.ldap.internal.LdapDirectory.Outcome;
import org.globalart.astrum.ssl.TrustStore;

import org.junit.AfterClass;
import org.junit.BeforeClass;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertThrows;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class LdapDirectoryTest
{
  private static LdapTestDirectory ldap;

  private static LdapDirectory underTest;

  @BeforeClass
  public static void start() throws Exception {
    ldap = new LdapTestDirectory();
    TrustStore trustStore = mock(TrustStore.class);
    when(trustStore.getSSLContext()).thenReturn(ldap.trustingContext());
    underTest = new LdapDirectory(() -> trustStore);
  }

  @AfterClass
  public static void stop() {
    ldap.close();
  }

  @Test
  public void verifyConnectionWithSystemAccount() {
    underTest.verifyConnection(ldap.dynamicServer("a"), LdapTestDirectory.systemPassword());
  }

  @Test
  public void verifyConnectionAnonymously() {
    LdapServer server = ldap.dynamicServer("a");
    server.setAuthScheme(LdapAuthScheme.NONE);
    underTest.verifyConnection(server, null);
  }

  @Test
  public void verifyConnectionRejectsWrongSystemPassword() {
    LdapServerException e = assertThrows(LdapServerException.class,
        () -> underTest.verifyConnection(ldap.dynamicServer("a"), "wrong".toCharArray()));
    assertThat(e.getMessage(), containsString("rejected"));
  }

  @Test
  public void verifyConnectionNeverBindsWithAnEmptySystemPassword() {
    LdapServerException e = assertThrows(LdapServerException.class,
        () -> underTest.verifyConnection(ldap.dynamicServer("a"), new char[0]));
    assertThat(e.getMessage(), containsString("No password"));
  }

  @Test
  public void verifyConnectionReportsMissingSearchBase() {
    LdapServer server = ldap.dynamicServer("a");
    server.setSearchBase("ou=missing," + LdapTestDirectory.BASE);
    LdapServerException e = assertThrows(LdapServerException.class,
        () -> underTest.verifyConnection(server, LdapTestDirectory.systemPassword()));
    assertThat(e.getMessage(), containsString("does not exist"));
  }

  @Test
  public void verifyConnectionReportsUnreachableServer() {
    LdapServer server = ldap.dynamicServer("a");
    server.setPort(1);
    LdapServerException e = assertThrows(LdapServerException.class,
        () -> underTest.verifyConnection(server, LdapTestDirectory.systemPassword()));
    assertThat(e.getMessage(), containsString("Could not connect"));
  }

  @Test
  public void ldapsWithTrustStore() {
    LdapServer server = ldaps();
    server.setUseTrustStore(true);
    assertThat(underTest.findUser(server, LdapTestDirectory.systemPassword(), "alice").isPresent(), is(true));
  }

  @Test
  public void ldapsWithoutTrustStoreRejectsUnknownCertificate() {
    LdapServer server = ldaps();
    server.setUseTrustStore(false);
    assertThrows(LdapServerException.class,
        () -> underTest.verifyConnection(server, LdapTestDirectory.systemPassword()));
  }

  @Test
  public void dynamicGroupsComeFromMemberOf() {
    LdapUser alice = underTest.findUser(ldap.dynamicServer("a"), LdapTestDirectory.systemPassword(), "alice")
        .orElseThrow();
    assertThat(alice.username(), is("alice"));
    assertThat(alice.realName(), is("Alice Smith"));
    assertThat(alice.email(), is("alice@example.com"));
    assertThat(alice.dn(), is("uid=alice,ou=people," + LdapTestDirectory.BASE));
    assertThat(alice.groups(), containsInAnyOrder("developers", "admins"));
  }

  @Test
  public void staticGroupsComeFromGroupEntries() {
    LdapServer server = ldap.staticServer("a");
    assertThat(underTest.findUser(server, LdapTestDirectory.systemPassword(), "alice").orElseThrow().groups(),
        containsInAnyOrder("developers", "admins"));
    assertThat(underTest.findUser(server, LdapTestDirectory.systemPassword(), "bob").orElseThrow().groups(),
        contains("developers"));
    assertThat(underTest.findUser(server, LdapTestDirectory.systemPassword(), "carol").orElseThrow().groups(),
        empty());
  }

  @Test
  public void staticGroupsWithUsernameFormat() {
    LdapServer server = ldap.staticServer("a");
    server.setGroupMemberFormat("uid=${username},ou=people," + LdapTestDirectory.BASE);
    assertThat(underTest.findUser(server, LdapTestDirectory.systemPassword(), "bob").orElseThrow().groups(),
        contains("developers"));
  }

  @Test
  public void groupsAreLeftOutUnlessMapped() {
    LdapServer server = ldap.dynamicServer("a");
    server.setLdapGroupsAsRoles(false);
    assertThat(underTest.findUser(server, LdapTestDirectory.systemPassword(), "alice").orElseThrow().groups(),
        empty());
  }

  @Test
  public void userLookupIsCaseInsensitiveAndKeepsTheStoredName() {
    Optional<LdapUser> user = underTest.findUser(ldap.dynamicServer("a"), LdapTestDirectory.systemPassword(), "ALICE");
    assertThat(user.orElseThrow().username(), is("alice"));
  }

  @Test
  public void unknownUser() {
    assertThat(underTest.findUser(ldap.dynamicServer("a"), LdapTestDirectory.systemPassword(), "nobody").isPresent(),
        is(false));
  }

  @Test
  public void userSubtree() {
    LdapServer server = ldap.dynamicServer("a");
    assertThat(underTest.findUser(server, LdapTestDirectory.systemPassword(), "dave").isPresent(), is(false));
    server.setUserSubtree(true);
    assertThat(underTest.findUser(server, LdapTestDirectory.systemPassword(), "dave").isPresent(), is(true));
  }

  @Test
  public void userFilterWithoutParentheses() {
    LdapServer server = ldap.dynamicServer("a");
    server.setUserLdapFilter("mail=*@example.com");
    assertThat(underTest.findUser(server, LdapTestDirectory.systemPassword(), "carol").isPresent(), is(false));
    assertThat(underTest.findUser(server, LdapTestDirectory.systemPassword(), "alice").isPresent(), is(true));
  }

  @Test
  public void filterValuesAreEscaped() {
    assertThat(underTest.findUser(ldap.dynamicServer("a"), LdapTestDirectory.systemPassword(), "*").isPresent(),
        is(false));
  }

  @Test
  public void authenticateByBinding() {
    LdapServer server = ldap.dynamicServer("a");
    Authentication ok = underTest.authenticate(server, LdapTestDirectory.systemPassword(), "alice",
        "alice-secret".toCharArray());
    assertThat(ok.outcome(), is(Outcome.AUTHENTICATED));
    assertThat(ok.user().groups(), containsInAnyOrder("developers", "admins"));
    assertThat(underTest.authenticate(server, LdapTestDirectory.systemPassword(), "alice", "wrong".toCharArray())
        .outcome(), is(Outcome.INCORRECT_PASSWORD));
    assertThat(underTest.authenticate(server, LdapTestDirectory.systemPassword(), "nobody", "x".toCharArray())
        .outcome(), is(Outcome.USER_NOT_FOUND));
  }

  @Test
  public void emptyPasswordNeverAuthenticates() {
    LdapServer server = ldap.dynamicServer("a");
    assertThat(underTest.authenticate(server, LdapTestDirectory.systemPassword(), "alice", new char[0]).outcome(),
        is(Outcome.INCORRECT_PASSWORD));
    assertThat(underTest.authenticate(server, LdapTestDirectory.systemPassword(), "alice", null).outcome(),
        is(Outcome.INCORRECT_PASSWORD));
  }

  @Test
  public void authenticateByComparingThePasswordAttribute() {
    LdapServer server = ldap.dynamicServer("a");
    server.setUserPasswordAttribute("userPassword");
    assertThat(underTest.authenticate(server, LdapTestDirectory.systemPassword(), "bob", "bob-secret".toCharArray())
        .outcome(), is(Outcome.AUTHENTICATED));
    assertThat(underTest.authenticate(server, LdapTestDirectory.systemPassword(), "bob", "bob-Secret".toCharArray())
        .outcome(), is(Outcome.INCORRECT_PASSWORD));
    assertThat(underTest.authenticate(server, LdapTestDirectory.systemPassword(), "carol",
        "carol-secret".toCharArray()).outcome(), is(Outcome.AUTHENTICATED));
  }

  @Test
  public void searchUsersByPrefix() {
    LdapServer server = ldap.dynamicServer("a");
    List<LdapUser> all = underTest.searchUsers(server, LdapTestDirectory.systemPassword(), null, 100);
    assertThat(all.stream().map(LdapUser::username).toList(), containsInAnyOrder("alice", "bob", "carol"));
    List<LdapUser> some = underTest.searchUsers(server, LdapTestDirectory.systemPassword(), "a", 100);
    assertThat(some.stream().map(LdapUser::username).toList(), contains("alice"));
    assertThat(underTest.searchUsers(server, LdapTestDirectory.systemPassword(), null, 1).size(), is(1));
  }

  @Test
  public void searchStaticGroups() {
    LdapServer server = ldap.staticServer("a");
    assertThat(underTest.searchGroups(server, LdapTestDirectory.systemPassword(), null, 100),
        containsInAnyOrder("developers", "admins"));
    assertThat(underTest.searchGroups(server, LdapTestDirectory.systemPassword(), "dev", 100),
        contains("developers"));
    assertThat(underTest.groupExists(server, LdapTestDirectory.systemPassword(), "admins"), is(true));
    assertThat(underTest.groupExists(server, LdapTestDirectory.systemPassword(), "nobody"), is(false));
  }

  @Test
  public void searchDynamicGroups() {
    LdapServer server = ldap.dynamicServer("a");
    assertThat(underTest.searchGroups(server, LdapTestDirectory.systemPassword(), "min", 100), contains("admins"));
    assertThat(underTest.groupExists(server, LdapTestDirectory.systemPassword(), "developers"), is(true));
  }

  @Test
  public void groupName() {
    assertThat(LdapDirectory.groupName("cn=developers,ou=groups,dc=example,dc=com"), is("developers"));
    assertThat(LdapDirectory.groupName("CN=Domain Users,CN=Users,DC=corp,DC=local"), is("Domain Users"));
    assertThat(LdapDirectory.groupName("not a dn"), is("not a dn"));
  }

  @Test
  public void bracketedIpv6Host() {
    LdapServer server = ldap.dynamicServer("a");
    server.setHost("[::1]");
    assertThat(LdapDirectory.host(server), is("::1"));
    assertThat(Set.of(LdapDirectory.host(ldap.dynamicServer("b"))), contains("localhost"));
  }

  private LdapServer ldaps() {
    LdapServer server = ldap.dynamicServer("secure");
    server.setProtocol(LdapProtocol.LDAPS);
    server.setPort(ldap.ldapsPort());
    return server;
  }
}
