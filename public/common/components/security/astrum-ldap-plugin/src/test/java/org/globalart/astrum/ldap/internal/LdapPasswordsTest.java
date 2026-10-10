package org.globalart.astrum.ldap.internal;

import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.util.Base64;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class LdapPasswordsTest
{
  @Test
  public void plainText() {
    assertThat(matches("secret", "secret"), is(true));
    assertThat(matches("secret", "Secret"), is(false));
    assertThat(matches("", ""), is(false));
  }

  @Test
  public void saltedSha() throws Exception {
    assertThat(matches("bob-secret", LdapTestDirectory.ssha("bob-secret")), is(true));
    assertThat(matches("bob-secreT", LdapTestDirectory.ssha("bob-secret")), is(false));
  }

  @Test
  public void unsaltedHashes() throws Exception {
    assertThat(matches("pw", "{SHA}" + digest("SHA-1", "pw")), is(true));
    assertThat(matches("pw", "{sha256}" + digest("SHA-256", "pw")), is(true));
    assertThat(matches("pw", "{SHA512}" + digest("SHA-512", "pw")), is(true));
    assertThat(matches("pw", "{MD5}" + digest("MD5", "pw")), is(true));
    assertThat(matches("px", "{SHA}" + digest("SHA-1", "pw")), is(false));
  }

  @Test
  public void unsupportedOrBrokenValues() {
    assertThat(matches("pw", "{CRYPT}abcdefgh"), is(false));
    assertThat(matches("pw", "{SSHA}not base64!"), is(false));
    assertThat(matches("pw", "{SHA}"), is(false));
  }

  private static boolean matches(final String password, final String stored) {
    return LdapPasswords.matches(password.toCharArray(), stored.getBytes(StandardCharsets.UTF_8));
  }

  private static String digest(final String algorithm, final String value) throws Exception {
    return Base64.getEncoder().encodeToString(
        MessageDigest.getInstance(algorithm).digest(value.getBytes(StandardCharsets.UTF_8)));
  }
}
