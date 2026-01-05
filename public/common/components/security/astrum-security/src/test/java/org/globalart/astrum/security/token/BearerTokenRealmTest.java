
package org.globalart.astrum.security.token;

import java.security.Principal;
import java.util.Optional;
import jakarta.inject.Provider;
import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.security.UserPrincipalsHelper;
import org.globalart.astrum.security.authc.AstrumApiKeyAuthenticationToken;
import org.globalart.astrum.security.authc.apikey.ApiKey;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;
import org.globalart.astrum.security.user.UserNotFoundException;
import org.globalart.astrum.security.user.UserStatus;

import org.apache.shiro.authc.AuthenticationInfo;
import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.authc.credential.CredentialsMatcher;
import org.apache.shiro.subject.PrincipalCollection;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.Is.is;
import static org.hamcrest.core.IsEqual.equalTo;
import static org.hamcrest.core.IsNull.notNullValue;
import static org.hamcrest.core.IsNull.nullValue;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.security.token.BearerTokenRealm.ANONYMOUS_USER;
import static org.globalart.astrum.security.token.BearerTokenRealm.IS_TOKEN_AUTH_KEY;

@RunWith(MockitoJUnitRunner.Silent.class)
public class BearerTokenRealmTest
{
  private static final String FORMAT = "format";

  @Mock
  private ApiKeyService keyStore;

  @Mock
  private UserPrincipalsHelper principalsHelper;

  @Mock
  private AstrumApiKeyAuthenticationToken token;

  @Mock
  private AuthenticationToken unsupportedToken;

  @Mock
  private PrincipalCollection principalCollection;

  @Mock
  private Principal principal;

  @Mock
  private Provider<HttpServletRequest> requestProvider;

  @Mock
  private HttpServletRequest request;

  @Mock
  private CredentialsMatcher credentialsMatcher;

  BearerTokenRealm underTest;

  @Before
  public void setup() throws Exception {
    when(token.getPrincipal()).thenReturn(FORMAT);
    when(unsupportedToken.getPrincipal()).thenReturn(FORMAT);
    when(principalCollection.getPrimaryPrincipal()).thenReturn(principal);
    ApiKey key = mock(ApiKey.class);
    when(key.getPrincipals()).thenReturn(principalCollection);
    when(keyStore.getApiKeyByToken(any(), any())).thenReturn(Optional.of(key));
    when(principalsHelper.getUserStatus(principalCollection)).thenReturn(UserStatus.active);
    when(credentialsMatcher.doCredentialsMatch(any(), any())).thenReturn(true);
    when(requestProvider.get()).thenReturn(request);
    underTest = new BearerTokenRealm(keyStore, principalsHelper, FORMAT)
    {
    };
    underTest.setRequestProvider(requestProvider);
    underTest.setCredentialsMatcher(credentialsMatcher);
  }

  @Test
  public void supportedWhenCorrectTypeAndFormat() throws Exception {
    assertTrue(underTest.supports(token));
  }

  @Test
  public void notSupportedWhenWrongType() throws Exception {
    assertFalse(underTest.supports(unsupportedToken));
  }

  @Test
  public void notSupportedWhenWrongFormat() throws Exception {
    when(token.getPrincipal()).thenReturn("UnsupportedFormat");
    assertFalse(underTest.supports(token));
  }

  @Test
  public void getAuthInfoWhenActive() throws Exception {
    AuthenticationInfo authenticationInfo = underTest.doGetAuthenticationInfo(token);
    assertThat(authenticationInfo.getPrincipals(), is(notNullValue()));
  }

  @Test
  public void getAuthInfoWhenAnonymousAndSupported() throws Exception {
    when(principalCollection.getPrimaryPrincipal()).thenReturn(ANONYMOUS_USER);
    when(principalsHelper.getUserStatus(principalCollection)).thenReturn(UserStatus.disabled);
    underTest = new BearerTokenRealm(keyStore, principalsHelper, FORMAT)
    {
      @Override
      protected boolean isAnonymousSupported() {
        return true;
      }
    };
    AuthenticationInfo authenticationInfo = underTest.doGetAuthenticationInfo(token);
    assertThat(authenticationInfo.getPrincipals(), is(notNullValue()));
  }

  @Test
  public void nullWhenAnonymousButNotSupported() throws Exception {
    when(principalsHelper.getUserStatus(principalCollection)).thenReturn(UserStatus.disabled);
    when(principalCollection.getPrimaryPrincipal()).thenReturn(ANONYMOUS_USER);
    assertThat(underTest.doGetAuthenticationInfo(token), is(nullValue()));
  }

  @Test
  public void deleteKeysOnUserNotFoundException() throws Exception {
    when(principalsHelper.getUserStatus(principalCollection)).thenThrow(new UserNotFoundException("userid"));
    assertThat(underTest.doGetAuthenticationInfo(token), is(nullValue()));
    verify(keyStore).deleteApiKeys(principalCollection);
  }

  @Test
  public void nullAuthInfoWhenPrincipalsNull() throws Exception {
    when(keyStore.getApiKeyByToken(any(), any())).thenReturn(Optional.empty());
    assertThat(underTest.doGetAuthenticationInfo(token), is(nullValue()));
  }

  @Test
  public void nullAuthInfoWhenUserNotActive() throws Exception {
    when(principalsHelper.getUserStatus(principalCollection)).thenReturn(UserStatus.disabled);
    assertThat(underTest.doGetAuthenticationInfo(token), is(nullValue()));
  }

  @Test
  public void primaryPrincipalWhenGetCacheKey() throws Exception {
    assertThat(underTest.getAuthenticationCacheKey(token), is(equalTo(principal)));
  }

  @Test
  public void nullWhenTokenNull() throws Exception {
    assertThat(underTest.getAuthenticationCacheKey(null), is(nullValue()));
  }

  @Test
  public void nullWhenPrincipalsNull() throws Exception {
    when(keyStore.getApiKeyByToken(any(), any())).thenReturn(Optional.empty());
    assertThat(underTest.getAuthenticationCacheKey(token), is(nullValue()));
  }

  @Test
  public void anonymousAccessNotSupportedByDefault() throws Exception {
    assertThat(underTest.isAnonymousSupported(), is(equalTo(false)));
  }

  @Test
  public void cachingEnabled() {
    assertThat(underTest.isAuthenticationCachingEnabled(), is(true));
  }

  @Test
  public void verifyAssertCredentialsMatchSetsAttributes() {
    underTest.assertCredentialsMatch(token, underTest.doGetAuthenticationInfo(token));
    verify(request).setAttribute(IS_TOKEN_AUTH_KEY, Boolean.TRUE);
    verify(token).setPrincipal(principal);
  }
}
