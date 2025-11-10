
package org.globalart.astrum.crypto.secrets;

import com.google.common.collect.ImmutableMap;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertThrows;

public class EncryptedSecretTest
{
  @Test
  public void testCreatesParsePhcSecretWithVersion() {
    EncryptedSecret secret =
        new EncryptedSecret("test", "1.0", "test-salt", "value", ImmutableMap.of("key1", "val1", "key2", "val2"));

    String phcString = secret.toPhcString();

    assertNotNull(phcString);
    assertThat(secret, equalTo(EncryptedSecret.parse(phcString)));
  }

  @Test
  public void testCreatesParsePhcSecretWithoutVersion() {
    EncryptedSecret secret =
        new EncryptedSecret("test2", null, "test-salt2", "value2", ImmutableMap.of("kv1", "v1", "kv2", "v2"));

    String phcString = secret.toPhcString();

    assertNotNull(phcString);
    assertThat(EncryptedSecret.parse(phcString), equalTo(secret));
  }

  @Test
  public void testParseInvalidPhcString() {
    String phcInvalid1 = "$AES$key1=val1,key2=val2,key3=val3$encryptedValue";
    String phcInvalid2 = "$this$is$a$test$failure";
    String phcInvalid3 = "";

    assertThrows(IllegalArgumentException.class, () -> EncryptedSecret.parse(phcInvalid1));
    assertThrows(IllegalArgumentException.class, () -> EncryptedSecret.parse(phcInvalid2));
    assertThrows(IllegalArgumentException.class, () -> EncryptedSecret.parse(phcInvalid3));

    // null phc string
    assertThrows(NullPointerException.class, () -> EncryptedSecret.parse(null));
  }
}
