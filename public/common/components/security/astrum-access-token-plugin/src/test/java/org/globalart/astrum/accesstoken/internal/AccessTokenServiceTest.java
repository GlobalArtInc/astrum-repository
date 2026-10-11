package org.globalart.astrum.accesstoken.internal;

import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import java.time.ZoneOffset;
import java.util.Optional;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.security.accesstoken.AccessTokenPrincipal;
import org.globalart.astrum.security.accesstoken.AccessTokenScope;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserDeletedEvent;
import org.globalart.astrum.testdb.DataSessionRule;

import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.subject.SimplePrincipalCollection;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.Matchers.startsWith;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;

public class AccessTokenServiceTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(AccessTokenDAO.class);

  private final EventManager eventManager = mock(EventManager.class);

  private final ApiKeyService apiKeyService = mock(ApiKeyService.class);

  private final MutableClock clock = new MutableClock(Instant.parse("2026-10-11T10:00:00Z"));

  private AccessTokenStore store;

  private AccessTokenService underTest;

  private final PrincipalCollection alice = new SimplePrincipalCollection("alice", "AstrumAuthenticatingRealm");

  @Before
  public void setUp() {
    store = new AccessTokenStore(sessionRule);
    store.setDependencies(eventManager);
    underTest = new AccessTokenService(store, eventManager, apiKeyService);
    underTest.setClock(clock);
  }

  @Test
  public void createdTokenAuthenticatesAndOnlyItsHashIsStored() {
    AccessTokenService.CreatedAccessToken created = underTest.create(alice, " ci ", AccessTokenScope.READ, 30);

    assertThat(created.secret(), startsWith(AccessTokenService.PREFIX));
    assertThat(created.secret().length(), is(AccessTokenService.PREFIX.length() + AccessTokenService.SECRET_LENGTH));
    AccessTokenData stored = store.read(created.token().getId()).orElseThrow();
    assertThat(stored.getName(), is("ci"));
    assertThat(stored.getUsername(), is("alice"));
    assertThat(stored.getTokenHash(), not(created.secret()));
    assertThat(stored.getTokenHint(), is(created.secret().substring(0, 11)));
    assertThat(stored.getExpires().toInstant(), is(Instant.parse("2026-11-10T10:00:00Z")));
    assertThat(stored.getPrincipals().getPrimaryPrincipal(), is("alice"));

    Optional<AccessTokenData> authenticated = underTest.authenticate(created.secret().toCharArray());
    assertThat(authenticated.map(AccessTokenData::getId), is(Optional.of(created.token().getId())));
    assertThat(underTest.authenticate((created.secret() + "x").toCharArray()), is(Optional.empty()));
    assertThat(underTest.authenticate("password".toCharArray()), is(Optional.empty()));
    assertThat(underTest.list("alice"), hasSize(1));
    assertThat(underTest.list("bob"), empty());
    verify(eventManager).post(any(AccessTokenEvent.class));
  }

  @Test
  public void namesAndLifetimesAreValidated() {
    underTest.create(alice, "ci", AccessTokenScope.READ, null);

    assertThrows(AccessTokenException.class, () -> underTest.create(alice, "ci", AccessTokenScope.WRITE, null));
    assertThrows(AccessTokenException.class, () -> underTest.create(alice, "  ", AccessTokenScope.READ, null));
    assertThrows(AccessTokenException.class, () -> underTest.create(alice, "x".repeat(101), AccessTokenScope.READ, 1));
    assertThrows(AccessTokenException.class, () -> underTest.create(alice, "zero", AccessTokenScope.READ, 0));
    assertThrows(AccessTokenException.class, () -> underTest.create(alice, "long", AccessTokenScope.READ, 3651));
    underTest.create(new SimplePrincipalCollection("bob", "AstrumAuthenticatingRealm"), "ci", AccessTokenScope.READ,
        null);
  }

  @Test
  public void expiredTokenNoLongerAuthenticates() {
    String secret = underTest.create(alice, "ci", AccessTokenScope.READ, 1).secret();
    assertThat(underTest.authenticate(secret.toCharArray()).isPresent(), is(true));

    clock.advance(Duration.ofDays(1));

    assertThat(underTest.authenticate(secret.toCharArray()).isPresent(), is(false));
  }

  @Test
  public void lastUseIsRecordedEveryFewMinutes() {
    AccessTokenService.CreatedAccessToken created = underTest.create(alice, "ci", AccessTokenScope.READ, null);
    assertThat(store.read(created.token().getId()).orElseThrow().getLastUsed(), nullValue());

    underTest.authenticate(created.secret().toCharArray());
    assertThat(store.read(created.token().getId()).orElseThrow().getLastUsed().toInstant(), is(clock.instant()));

    Instant firstUse = clock.instant();
    clock.advance(Duration.ofMinutes(1));
    underTest.authenticate(created.secret().toCharArray());
    assertThat(store.read(created.token().getId()).orElseThrow().getLastUsed().toInstant(), is(firstUse));

    clock.advance(Duration.ofMinutes(5));
    underTest.authenticate(created.secret().toCharArray());
    assertThat(store.read(created.token().getId()).orElseThrow().getLastUsed().toInstant(), is(clock.instant()));
  }

  @Test
  public void revokedTokenStopsWorkingAtOnceAndTakesIssuedKeysWithIt() {
    AccessTokenService.CreatedAccessToken created = underTest.create(alice, "ci", AccessTokenScope.WRITE, null);
    assertThat(underTest.authenticate(created.secret().toCharArray()).isPresent(), is(true));
    AccessTokenData token = underTest.read(created.token().getId()).orElseThrow();

    assertThat(underTest.revoke(token), is(true));

    assertThat(underTest.authenticate(created.secret().toCharArray()).isPresent(), is(false));
    assertThat(underTest.read(token.getId()).isPresent(), is(false));
    verify(apiKeyService).deleteApiKeys(AccessTokenService.principals(token));
  }

  @Test
  public void principalsCarryTheOwnerFirstAndTheScope() {
    AccessTokenData token = underTest.read(underTest.create(alice, "ci", AccessTokenScope.READ, null).token().getId())
        .orElseThrow();

    PrincipalCollection principals = AccessTokenService.principals(token);

    assertThat(principals.getPrimaryPrincipal(), is("alice"));
    assertThat(principals.getRealmNames(), contains("AstrumAuthenticatingRealm"));
    AccessTokenPrincipal scope = principals.oneByType(AccessTokenPrincipal.class);
    assertThat(scope, notNullValue());
    assertThat(scope.scope(), is("read"));
    assertThat(scope.tokenName(), is("ci"));
  }

  @Test
  public void deletingTheUserDeletesTheirTokens() {
    String secret = underTest.create(alice, "ci", AccessTokenScope.READ, null).secret();
    assertThat(underTest.authenticate(secret.toCharArray()).isPresent(), is(true));
    User user = new User();
    user.setUserId("alice");

    underTest.on(new UserDeletedEvent(user));

    assertThat(underTest.list("alice"), empty());
    assertThat(underTest.authenticate(secret.toCharArray()).isPresent(), is(false));
  }

  private static final class MutableClock
      extends Clock
  {
    private Instant now;

    private MutableClock(final Instant now) {
      this.now = now;
    }

    void advance(final Duration duration) {
      now = now.plus(duration);
    }

    @Override
    public ZoneOffset getZone() {
      return ZoneOffset.UTC;
    }

    @Override
    public Clock withZone(final java.time.ZoneId zone) {
      return this;
    }

    @Override
    public Instant instant() {
      return now;
    }
  }
}
