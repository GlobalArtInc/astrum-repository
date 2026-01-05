
package org.globalart.astrum.security.internal;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * Tests for {@link LegacyAstrumPasswordService}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class LegacyAstrumPasswordServiceTest
{
  private LegacyAstrumPasswordService underTest;

  @Before
  public void setUp() throws Exception {
    underTest = new LegacyAstrumPasswordService();
  }

  @Test
  public void testSha1Hash() {
    String password = "admin123";
    String sha1Hash = "f865b53623b121fd34ee5426c792e5c33af8c227";

    assertThat(underTest.passwordsMatch(password, sha1Hash), is(true));
  }

  @Test
  public void testMd5Hash() {
    String password = "admin123";
    String md5Hash = "0192023a7bbd73250516f069df18b500";

    assertThat(underTest.passwordsMatch(password, md5Hash), is(true));
  }

  @Test
  public void testInvalidSha1Hash() {
    String password = "admin123";
    String sha1Hash = "f865b53623b121fd34ee5426c792e5c33af8c228";

    assertThat(underTest.passwordsMatch(password, sha1Hash), is(false));
  }

  @Test
  public void testInvalidMd5Hash() {
    String password = "admin123";
    String md5Hash = "0192023a7bbd73250516f069df18b501";

    assertThat(underTest.passwordsMatch(password, md5Hash), is(false));
  }

  @Test
  public void testNullEncryptedReturnsFalse() {
    assertThat(underTest.passwordsMatch("admin123", null), is(false));
  }

  @Test
  public void testNullPlaintextReturnsFalse() {
    String sha1Hash = "f865b53623b121fd34ee5426c792e5c33af8c227";

    assertThat(underTest.passwordsMatch(null, sha1Hash), is(false));
  }
}
