package org.globalart.astrum.repository.terraform.internal.signing;

import java.util.Map;

import org.junit.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

public class Sha256SumsTest
{
  private static final String A = "a".repeat(64);

  private static final String B = "B".repeat(64);

  @Test
  public void parsesTheOutputOfSha256sum() {
    Sha256Sums sums = Sha256Sums.parse((A + "  terraform-provider-demo_1.0.0_linux_amd64.zip\n"
        + B + " *terraform-provider-demo_1.0.0_darwin_arm64.zip\r\n\n").getBytes(UTF_8));

    assertThat(sums.digest("terraform-provider-demo_1.0.0_linux_amd64.zip")).contains(A);
    assertThat(sums.digest("terraform-provider-demo_1.0.0_darwin_arm64.zip")).contains("b".repeat(64));
    assertThat(sums.digest("missing.zip")).isEmpty();
  }

  @Test
  public void writesSortedLinesWithTwoSpaces() {
    byte[] document = Sha256Sums.of(Map.of("z.zip", A, "a.zip", B)).toBytes();

    assertThat(new String(document, UTF_8)).isEqualTo("b".repeat(64) + "  a.zip\n" + A + "  z.zip\n");
    assertThat(Sha256Sums.parse(document).digests()).hasSize(2);
  }

  @Test
  public void rejectsOtherDocuments() {
    assertThatThrownBy(() -> Sha256Sums.parse("hello world\n".getBytes(UTF_8)))
        .isInstanceOf(IllegalArgumentException.class)
        .hasMessageContaining("Line 1");
    assertThatThrownBy(() -> Sha256Sums.parse("\n".getBytes(UTF_8))).isInstanceOf(IllegalArgumentException.class);
  }
}
