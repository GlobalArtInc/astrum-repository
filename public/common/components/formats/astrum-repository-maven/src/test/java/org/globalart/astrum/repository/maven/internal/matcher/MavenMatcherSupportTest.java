
package org.globalart.astrum.repository.maven.internal.matcher;

import com.google.common.base.Predicate;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.IsEqual.equalTo;

/**
 * UT for {@link MavenMatcherSupport}
 *
 * @since 3.0
 */
public class MavenMatcherSupportTest

{
  @Test
  public void equalsWithHashes() {
    Predicate<String> pred = MavenMatcherSupport.withHashes("/some-path.txt"::equals);

    assertThat(pred.apply("/some-path.txt"), equalTo(true));
    assertThat(pred.apply("/some-path.txt.sha1"), equalTo(true));
    assertThat(pred.apply("/some-path.txt.md5"), equalTo(true));
    assertThat(pred.apply("/some-path.txt.sha256"), equalTo(true));
    assertThat(pred.apply("/some-path.txt.sha512"), equalTo(true));

    assertThat(pred.apply("some-path.txt"), equalTo(false));
    assertThat(pred.apply("/some-path.txt.crc"), equalTo(false));
    assertThat(pred.apply("/some-path.tx"), equalTo(false));
    assertThat(pred.apply("/other-path.txt"), equalTo(false));
  }

  @Test
  public void endsWithWithHashes() {
    Predicate<String> pred = MavenMatcherSupport.withHashes((String input) -> input.endsWith("/some-path.txt"));

    assertThat(pred.apply("/some/prefix/some-path.txt"), equalTo(true));
    assertThat(pred.apply("/some-path.txt.sha1"), equalTo(true));
    assertThat(pred.apply("some/prefix/some-path.txt.md5"), equalTo(true));
    assertThat(pred.apply("/some-path.txt.sha256"), equalTo(true));
    assertThat(pred.apply("/some-path.txt.sha512"), equalTo(true));

    assertThat(pred.apply("some-path.txt"), equalTo(false));
    assertThat(pred.apply("/some-path.txt.crc"), equalTo(false));
    assertThat(pred.apply("/some-path.tx"), equalTo(false));
    assertThat(pred.apply("/other-path.txt"), equalTo(false));
  }
}
