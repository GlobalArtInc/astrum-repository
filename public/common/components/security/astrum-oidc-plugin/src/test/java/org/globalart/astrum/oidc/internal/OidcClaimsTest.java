package org.globalart.astrum.oidc.internal;

import java.util.List;
import java.util.Map;

import org.junit.Test;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.CoreMatchers.nullValue;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.junit.Assert.assertThrows;

public class OidcClaimsTest
{
  private final Map<String, Object> claims = Map.of(
      "sub", "1486b1a2",
      "preferred_username", "roman",
      "email", "roman@example.com",
      "groups", List.of("as-admin", "developers"),
      "realm_access", Map.of("roles", List.of("deployer")),
      "https://example.com/team", "platform");

  @Test
  public void findsTopLevelNestedAndDottedNames() {
    assertThat(OidcClaims.string(claims, "email"), is("roman@example.com"));
    assertThat(OidcClaims.strings(claims, "realm_access.roles"), is(List.of("deployer")));
    assertThat(OidcClaims.string(claims, "https://example.com/team"), is("platform"));
    assertThat(OidcClaims.string(claims, "missing.claim"), is(nullValue()));
  }

  @Test
  public void readsGroupsFromListsAndSingleValues() {
    assertThat(OidcClaims.strings(claims, "groups"), is(List.of("as-admin", "developers")));
    assertThat(OidcClaims.strings(claims, "email"), is(List.of("roman@example.com")));
    assertThat(OidcClaims.strings(claims, "realm_access"), is(List.of()));
  }

  @Test
  public void usernameDefaultsToPreferredUsernameThenSubject() throws Exception {
    OidcConfiguration configuration = new OidcConfiguration();
    assertThat(OidcClaims.userId(configuration, claims), is("roman"));
    assertThat(OidcClaims.userId(configuration, Map.of("sub", "abc")), is("abc"));
  }

  @Test
  public void configuredUsernameClaimMustBePresent() throws Exception {
    OidcConfiguration configuration = new OidcConfiguration();
    configuration.setUsernameClaim("email");
    assertThat(OidcClaims.userId(configuration, claims), is("roman@example.com"));
    configuration.setUsernameClaim("upn");
    assertThrows(OidcException.class, () -> OidcClaims.userId(configuration, claims));
  }

  @Test
  public void exactMatchClaimsAcceptMatchingValuesAndListMembers() throws Exception {
    OidcConfiguration configuration = new OidcConfiguration();
    configuration.setExactMatchClaims(Map.of("email", "roman@example.com", "groups", "developers"));
    OidcClaims.checkExactMatch(configuration, claims);

    configuration.setExactMatchClaims(Map.of("groups", "auditors"));
    assertThrows(OidcException.class, () -> OidcClaims.checkExactMatch(configuration, claims));
  }
}
