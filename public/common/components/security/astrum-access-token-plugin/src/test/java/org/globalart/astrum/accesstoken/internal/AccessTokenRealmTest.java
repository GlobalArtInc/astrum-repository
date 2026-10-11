package org.globalart.astrum.accesstoken.internal;

import java.util.Optional;

import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.security.UserPrincipalsHelper;
import org.globalart.astrum.security.accesstoken.AccessTokenPrincipal;
import org.globalart.astrum.security.user.UserNotFoundException;
import org.globalart.astrum.security.user.UserStatus;

import org.apache.shiro.authc.AuthenticationInfo;
import org.apache.shiro.authc.BearerToken;
import org.apache.shiro.authc.DisabledAccountException;
import org.apache.shiro.authc.IncorrectCredentialsException;
import org.apache.shiro.authc.UnknownAccountException;
import org.apache.shiro.authc.UsernamePasswordToken;
import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.subject.SimplePrincipalCollection;
import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class AccessTokenRealmTest
{
  private static final String SECRET = AccessTokenService.PREFIX + "a".repeat(AccessTokenService.SECRET_LENGTH);

  private final AccessTokenService tokens = mock(AccessTokenService.class);

  private final UserPrincipalsHelper principalsHelper = mock(UserPrincipalsHelper.class);

  private final HttpServletRequest request = mock(HttpServletRequest.class);

  private AccessTokenRealm underTest;

  private AccessTokenData token;

  @Before
  public void setUp() throws Exception {
    underTest = new AccessTokenRealm(tokens, principalsHelper);
    underTest.setRequestProvider(() -> request);
    when(request.getRequestURI()).thenReturn("/repository/docker-hosted/v2/");

    token = new AccessTokenData();
    token.setId("id-1");
    token.setUsername("alice");
    token.setName("ci");
    token.setScope("read");
    token.setPrincipals(new SimplePrincipalCollection("alice", "LdapRealm"));
    when(tokens.authenticate(SECRET.toCharArray())).thenReturn(Optional.of(token));
    when(principalsHelper.getUserStatus(any())).thenReturn(UserStatus.active);
  }

  @Test
  public void supportsOnlyPasswordsShapedLikeTokens() {
    assertThat(underTest.supports(new UsernamePasswordToken("alice", SECRET)), is(true));
    assertThat(underTest.supports(new UsernamePasswordToken("alice", "secret")), is(false));
    assertThat(underTest.supports(new UsernamePasswordToken("alice", SECRET + "a")), is(false));
    assertThat(underTest.supports(new BearerToken(SECRET)), is(false));
  }

  @Test
  public void signsInAsTheOwnerNarrowedToTheTokenScope() {
    AuthenticationInfo info = underTest.getAuthenticationInfo(new UsernamePasswordToken("Alice", SECRET));

    PrincipalCollection principals = info.getPrincipals();
    assertThat(principals.getPrimaryPrincipal(), is("alice"));
    assertThat(principals.getRealmNames().size(), is(1));
    assertThat(principals.getRealmNames().iterator().next(), is("LdapRealm"));
    assertThat(principals.oneByType(AccessTokenPrincipal.class).scope(), is("read"));
  }

  @Test
  public void refusesUnknownTokensAndTokensOfOtherUsers() {
    when(tokens.authenticate(SECRET.toCharArray())).thenReturn(Optional.empty());
    assertThrows(IncorrectCredentialsException.class,
        () -> underTest.getAuthenticationInfo(new UsernamePasswordToken("alice", SECRET)));

    when(tokens.authenticate(SECRET.toCharArray())).thenReturn(Optional.of(token));
    assertThrows(IncorrectCredentialsException.class,
        () -> underTest.getAuthenticationInfo(new UsernamePasswordToken("bob", SECRET)));
  }

  @Test
  public void refusesInactiveAndRemovedUsers() throws Exception {
    when(principalsHelper.getUserStatus(any())).thenReturn(UserStatus.disabled);
    assertThrows(DisabledAccountException.class,
        () -> underTest.getAuthenticationInfo(new UsernamePasswordToken("alice", SECRET)));

    when(principalsHelper.getUserStatus(any())).thenThrow(new UserNotFoundException("alice"));
    assertThrows(UnknownAccountException.class,
        () -> underTest.getAuthenticationInfo(new UsernamePasswordToken("alice", SECRET)));
  }

  @Test
  public void doesNotStartUiSessions() {
    when(request.getRequestURI()).thenReturn("/service/rapture/session");

    assertThrows(IncorrectCredentialsException.class,
        () -> underTest.getAuthenticationInfo(new UsernamePasswordToken("alice", SECRET)));
  }
}
