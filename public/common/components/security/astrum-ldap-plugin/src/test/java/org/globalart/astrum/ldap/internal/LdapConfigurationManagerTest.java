package org.globalart.astrum.ldap.internal;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicInteger;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.rapture.PasswordPlaceholder;
import org.globalart.astrum.rest.ValidationErrorXO;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.security.realm.RealmManager;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.doAnswer;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

public class LdapConfigurationManagerTest
{
  private final List<LdapServer> saved = new ArrayList<>();

  private final AtomicInteger secrets = new AtomicInteger();

  private LdapConfigurationStore store;

  private RealmManager realmManager;

  private EventManager eventManager;

  private LdapConfigurationManager underTest;

  @Before
  public void setUp() {
    store = mock(LdapConfigurationStore.class);
    when(store.list()).thenAnswer(invocation -> saved.stream().map(LdapServer::copy).toList());
    doAnswer(invocation -> {
      List<LdapServer> servers = invocation.getArgument(0);
      saved.clear();
      servers.forEach(server -> saved.add(server.copy()));
      return null;
    }).when(store).save(any());
    when(store.encryptPassword(anyString(), any())).thenAnswer(invocation -> "secret-" + secrets.incrementAndGet());
    when(store.password(any())).thenAnswer(invocation -> ("stored:" + invocation.<LdapServer>getArgument(0)
        .getAuthPasswordId()).toCharArray());
    realmManager = mock(RealmManager.class);
    eventManager = mock(EventManager.class);
    underTest = new LdapConfigurationManager(store, realmManager, eventManager);
  }

  @Test
  public void createStoresTheEncryptedPasswordAndEnablesTheRealm() {
    LdapServer created = underTest.create(server("main"), "pw");

    assertThat(created.getId(), notNullValue());
    assertThat(saved.get(0).getAuthPasswordId(), is("secret-1"));
    verify(realmManager).enableRealm(LdapRealm.NAME);
    verify(eventManager).post(any(LdapConfigurationChangedEvent.class));
  }

  @Test
  public void createLeavesAnEnabledRealmAlone() {
    when(realmManager.isRealmEnabled(LdapRealm.NAME)).thenReturn(true);
    underTest.create(server("main"), "pw");
    verify(realmManager, never()).enableRealm(anyString());
  }

  @Test
  public void createRequiresAPasswordUnlessAnonymous() {
    assertThat(errorIds(() -> underTest.create(server("main"), "")), contains("authPassword"));
    assertThat(errorIds(() -> underTest.create(server("main"), PasswordPlaceholder.get())), contains("authPassword"));

    LdapServer anonymous = server("anonymous");
    anonymous.setAuthScheme(LdapAuthScheme.NONE);
    underTest.create(anonymous, null);
    assertThat(saved.get(0).getAuthPasswordId(), nullValue());
    assertThat(saved.get(0).getAuthUsername(), nullValue());
  }

  @Test
  public void namesAreUnique() {
    underTest.create(server("Main"), "pw");
    assertThat(errorIds(() -> underTest.create(server("main"), "pw")), contains("name"));
  }

  @Test
  public void validation() {
    LdapServer server = new LdapServer();
    server.setConnectionTimeoutSeconds(0);
    server.setMaxIncidentsCount(-1);
    server.setLdapGroupsAsRoles(true);
    server.setGroupType(LdapGroupType.STATIC);
    server.setUserLdapFilter("(&(broken");
    assertThat(errorIds(() -> underTest.create(server, "pw")), containsInAnyOrder(
        "name", "protocol", "host", "port", "searchBase", "authScheme", "connectionTimeoutSeconds", "maxIncidentsCount",
        "userObjectClass", "userLdapFilter", "userIdAttribute", "userRealNameAttribute", "userEmailAddressAttribute",
        "groupObjectClass", "groupIdAttribute", "groupMemberAttribute", "groupMemberFormat"));
  }

  @Test
  public void validationOfHostAndDn() {
    LdapServer server = server("main");
    server.setHost("ldap://example.com");
    server.setSearchBase("not a dn");
    assertThat(errorIds(() -> underTest.create(server, "pw")), containsInAnyOrder("host", "searchBase"));
    server.setHost("[::1]");
    server.setSearchBase("dc=example,dc=com");
    underTest.create(server, "pw");
  }

  @Test
  public void connectionValidationIgnoresTheMapping() {
    LdapServer server = server(null);
    server.setUserObjectClass(null);
    server.setUserIdAttribute(null);
    underTest.validateConnection(server);
  }

  @Test
  public void updateKeepsThePasswordWhenTheServerIsReachedTheSameWay() {
    String id = underTest.create(server("main"), "pw").getId();
    LdapServer changed = server("renamed");
    changed.setUserObjectClass("person");

    underTest.update(id, changed, "");
    underTest.update(id, changed, PasswordPlaceholder.get());

    assertThat(saved.get(0).getName(), is("renamed"));
    assertThat(saved.get(0).getAuthPasswordId(), is("secret-1"));
    verify(store, never()).removePassword("secret-1");
  }

  @Test
  public void updateToAnotherHostNeedsThePasswordAgain() {
    String id = underTest.create(server("main"), "pw").getId();
    LdapServer moved = server("main");
    moved.setHost("evil.example.com");

    assertThat(errorIds(() -> underTest.update(id, moved, null)), contains("authPassword"));

    underTest.update(id, moved, "new-pw");
    assertThat(saved.get(0).getAuthPasswordId(), is("secret-2"));
    verify(store).removePassword("secret-1");
  }

  @Test
  public void updateToAnonymousDropsThePassword() {
    String id = underTest.create(server("main"), "pw").getId();
    LdapServer anonymous = server("main");
    anonymous.setAuthScheme(LdapAuthScheme.NONE);
    underTest.update(id, anonymous, null);
    assertThat(saved.get(0).getAuthPasswordId(), nullValue());
    verify(store).removePassword("secret-1");
  }

  @Test
  public void updateOfAMissingServer() {
    assertThrows(LdapServerNotFoundException.class, () -> underTest.update("nope", server("main"), "pw"));
  }

  @Test
  public void deleteRemovesTheSecret() {
    String id = underTest.create(server("main"), "pw").getId();
    underTest.delete(id);
    assertThat(saved.isEmpty(), is(true));
    verify(store).removePassword("secret-1");
  }

  @Test
  public void changeOrder() {
    String a = underTest.create(server("a"), "pw").getId();
    String b = underTest.create(server("b"), "pw").getId();
    String c = underTest.create(server("c"), "pw").getId();

    underTest.changeOrder(List.of(c, a));

    assertThat(saved.stream().map(LdapServer::getName).toList(), contains("c", "a", "b"));
    assertThat(errorIds(() -> underTest.changeOrder(List.of("missing"))), contains("order"));
    assertThat(b, notNullValue());
  }

  @Test
  public void bindPasswordForVerification() {
    LdapServer existing = underTest.create(server("main"), "pw");
    LdapServer draft = server("main");

    assertThat(new String(underTest.bindPassword(draft, "typed", Optional.of(existing))), is("typed"));
    assertThat(new String(underTest.bindPassword(draft, "", Optional.of(existing))), is("stored:secret-1"));
    assertThat(errorIds(() -> underTest.bindPassword(draft, "", Optional.empty())), contains("authPassword"));

    draft.setAuthUsername("cn=someone-else");
    assertThat(errorIds(() -> underTest.bindPassword(draft, PasswordPlaceholder.get(), Optional.of(existing))),
        contains("authPassword"));

    draft.setAuthScheme(LdapAuthScheme.NONE);
    assertThat(underTest.bindPassword(draft, null, Optional.empty()), nullValue());
  }

  @Test
  public void normalize() {
    LdapServer server = server("  main ");
    server.setUseTrustStore(true);
    server.setUserBaseDn("  ");
    server.setAuthRealm("realm");
    server.setLdapGroupsAsRoles(false);

    LdapServer normalized = LdapConfigurationManager.normalize(server);

    assertThat(normalized.getName(), is("main"));
    assertThat(normalized.isUseTrustStore(), is(false));
    assertThat(normalized.getUserBaseDn(), nullValue());
    assertThat(normalized.getAuthRealm(), nullValue());
    assertThat(normalized.getGroupType(), nullValue());
  }

  static LdapServer server(final String name) {
    LdapServer server = new LdapServer();
    server.setName(name);
    server.setProtocol(LdapProtocol.LDAP);
    server.setHost("ldap.example.com");
    server.setPort(389);
    server.setSearchBase("dc=example,dc=com");
    server.setAuthScheme(LdapAuthScheme.SIMPLE);
    server.setAuthUsername("cn=svc,dc=example,dc=com");
    server.setUserObjectClass("inetOrgPerson");
    server.setUserIdAttribute("uid");
    server.setUserRealNameAttribute("cn");
    server.setUserEmailAddressAttribute("mail");
    server.setLdapGroupsAsRoles(true);
    server.setGroupType(LdapGroupType.DYNAMIC);
    server.setUserMemberOfAttribute("memberOf");
    return server;
  }

  private static List<String> errorIds(final Runnable runnable) {
    ValidationErrorsException e = assertThrows(ValidationErrorsException.class, runnable::run);
    return e.getValidationErrors().stream().map(ValidationErrorXO::getId).toList();
  }
}
