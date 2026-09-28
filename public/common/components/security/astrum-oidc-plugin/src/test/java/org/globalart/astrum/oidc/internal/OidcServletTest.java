package org.globalart.astrum.oidc.internal;

import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.LinkedHashMap;
import java.util.Map;

import org.junit.Test;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.CoreMatchers.nullValue;
import static org.hamcrest.MatcherAssert.assertThat;

public class OidcServletTest
{
  @Test
  public void codeChallengeIsTheUnpaddedBase64UrlSha256OfTheVerifier() {
    assertThat(OidcServlet.codeChallenge("dBjftJeZ4CVP-mJ92K0Q8tPOMzS2lX5nWGqOOv3fWZM"),
        is("rFdigAQdlo3oIMo8ZkXlufTjAjvHY0VXLZTNtSD0pn8"));
  }

  @Test
  public void returnHashIsDecodedFromBase64AndStripped() {
    String encoded = Base64.getEncoder().encodeToString("#browse/browse:maven-releases".getBytes(StandardCharsets.UTF_8));
    assertThat(OidcServlet.returnHash(encoded), is("browse/browse:maven-releases"));
    assertThat(OidcServlet.returnHash("#admin/security/users"), is("admin/security/users"));
    assertThat(OidcServlet.returnHash("YWRtaW4="), is("YWRtaW4="));
  }

  @Test
  public void returnHashRejectsControlCharactersAndEmptyValues() {
    String encoded = Base64.getEncoder().encodeToString("#browse\r\nSet-Cookie: x=y".getBytes(StandardCharsets.UTF_8));
    assertThat(OidcServlet.returnHash(encoded), is(nullValue()));
    assertThat(OidcServlet.returnHash("#browse\nx"), is(nullValue()));
    assertThat(OidcServlet.returnHash(""), is(nullValue()));
    assertThat(OidcServlet.returnHash(null), is(nullValue()));
  }

  @Test
  public void queryParametersAreEncodedAndAppended() {
    Map<String, String> parameters = new LinkedHashMap<>();
    parameters.put("scope", "openid profile");
    parameters.put("redirect_uri", "http://10.0.0.153:8081/oidc/callback");
    assertThat(OidcServlet.appendQuery("https://sso.example.com/authorize", parameters),
        is("https://sso.example.com/authorize?scope=openid+profile&redirect_uri=http%3A%2F%2F10.0.0.153%3A8081%2Foidc%2Fcallback"));
    assertThat(OidcServlet.appendQuery("https://sso.example.com/authorize?tenant=a", Map.of("x", "1")),
        is("https://sso.example.com/authorize?tenant=a&x=1"));
  }
}
