package org.globalart.astrum.oidc.internal;

import java.math.BigInteger;
import java.security.KeyPair;
import java.security.KeyPairGenerator;
import java.security.interfaces.RSAPrivateKey;
import java.security.interfaces.RSAPublicKey;
import java.time.Instant;
import java.util.Base64;
import java.util.Date;
import java.util.List;
import java.util.Map;

import com.auth0.jwt.JWT;
import com.auth0.jwt.algorithms.Algorithm;
import org.globalart.astrum.httpclient.HttpClientManager;
import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.junit.Assert.assertThrows;
import static org.mockito.Mockito.mock;

public class OidcProviderTest
{
  private static final String CLIENT_ID = "astrum";

  private KeyPair keyPair;

  private OidcConfiguration configuration;

  private OidcProvider underTest;

  @Before
  public void setUp() throws Exception {
    KeyPairGenerator generator = KeyPairGenerator.getInstance("RSA");
    generator.initialize(2048);
    keyPair = generator.generateKeyPair();
    RSAPublicKey publicKey = (RSAPublicKey) keyPair.getPublic();

    configuration = new OidcConfiguration();
    configuration.setClientId(CLIENT_ID);
    configuration.setIdpAuthorizationUrl("https://sso.example.com/authorize");
    configuration.setIdpTokenUrl("https://sso.example.com/token");
    configuration.setIdpJwks("{\"keys\":[{\"kty\":\"RSA\",\"use\":\"sig\",\"kid\":\"k1\",\"n\":\""
        + base64Url(publicKey.getModulus()) + "\",\"e\":\"" + base64Url(publicKey.getPublicExponent()) + "\"}]}");

    underTest = new OidcProvider(mock(HttpClientManager.class), () -> null);
  }

  @Test
  public void acceptsAValidTokenAndReturnsItsClaims() throws Exception {
    Map<String, Object> claims = underTest.verifyIdToken(configuration, token(CLIENT_ID, "n1", 300, "k1"), "n1");
    assertThat(claims.get("preferred_username"), is("roman"));
    assertThat(claims.get("groups"), is(List.of("as-admin")));
  }

  @Test
  public void rejectsAWrongNonceAudienceExpiryOrKey() {
    assertThrows(OidcException.class,
        () -> underTest.verifyIdToken(configuration, token(CLIENT_ID, "other", 300, "k1"), "n1"));
    assertThrows(OidcException.class,
        () -> underTest.verifyIdToken(configuration, token("someone-else", "n1", 300, "k1"), "n1"));
    assertThrows(OidcException.class,
        () -> underTest.verifyIdToken(configuration, token(CLIENT_ID, "n1", -600, "k1"), "n1"));
    assertThrows(OidcException.class,
        () -> underTest.verifyIdToken(configuration, token(CLIENT_ID, "n1", 300, "unknown"), "n1"));
  }

  @Test
  public void rejectsATokenSignedByAnotherKey() throws Exception {
    KeyPairGenerator generator = KeyPairGenerator.getInstance("RSA");
    generator.initialize(2048);
    KeyPair other = generator.generateKeyPair();
    String forged = JWT.create()
        .withKeyId("k1")
        .withAudience(CLIENT_ID)
        .withClaim("nonce", "n1")
        .withExpiresAt(Date.from(Instant.now().plusSeconds(300)))
        .sign(Algorithm.RSA256((RSAPublicKey) other.getPublic(), (RSAPrivateKey) other.getPrivate()));
    assertThrows(OidcException.class, () -> underTest.verifyIdToken(configuration, forged, "n1"));
  }

  @Test
  public void enforcesTheConfiguredAlgorithm() {
    configuration.setIdpJwsAlgorithm("ES256");
    assertThrows(OidcException.class,
        () -> underTest.verifyIdToken(configuration, token(CLIENT_ID, "n1", 300, "k1"), "n1"));
  }

  private String token(final String audience, final String nonce, final long expiresInSeconds, final String keyId) {
    return JWT.create()
        .withKeyId(keyId)
        .withSubject("1486b1a2")
        .withAudience(audience)
        .withClaim("nonce", nonce)
        .withClaim("preferred_username", "roman")
        .withArrayClaim("groups", new String[]{"as-admin"})
        .withIssuedAt(new Date())
        .withExpiresAt(Date.from(Instant.now().plusSeconds(expiresInSeconds)))
        .sign(Algorithm.RSA256((RSAPublicKey) keyPair.getPublic(), (RSAPrivateKey) keyPair.getPrivate()));
  }

  private static String base64Url(final BigInteger value) {
    byte[] bytes = value.toByteArray();
    if (bytes.length > 1 && bytes[0] == 0) {
      byte[] trimmed = new byte[bytes.length - 1];
      System.arraycopy(bytes, 1, trimmed, 0, trimmed.length);
      bytes = trimmed;
    }
    return Base64.getUrlEncoder().withoutPadding().encodeToString(bytes);
  }
}
