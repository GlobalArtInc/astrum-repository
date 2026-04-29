
package org.globalart.astrum.repository.r.internal.util;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.greaterThan;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.lessThan;
import static org.hamcrest.Matchers.not;

public class RPackageVersionTest
{
  private static final String[][] ORDERED = {
      {"0.0.2", "0.0.1"},
      {"0.2.0", "0.1.1"},
      {"1.2.0", "1.1.9"},
      {"2.0.0", "1.9.9"},
      {"10.0.0", "9.9999.9999"},
      {"0.0-2", "0.0-1"},
      {"1.2-0", "1.1-9"},
      {"10.0-0", "9.9-9999"},
      {"0.2.0", "0.1-1"},
      {"10.0-0", "9.9.9999"},
      {"1.10.0", "1.9.0"},
      {"1.0.0.1", "1.0.0"},
      {"3.6.3", "3.6.2.9000"}
  };

  @Test
  public void comparesGreaterAndLowerVersions() {
    for (String[] pair : ORDERED) {
      assertThat(pair[0] + " > " + pair[1], new RPackageVersion(pair[0]),
          is(greaterThan(new RPackageVersion(pair[1]))));
      assertThat(pair[1] + " < " + pair[0], new RPackageVersion(pair[1]),
          is(lessThan(new RPackageVersion(pair[0]))));
    }
  }

  @Test
  public void treatsDotsAndDashesAlike() {
    assertThat(new RPackageVersion("0.4-1"), is(equalTo(new RPackageVersion("0.4.1"))));
    assertThat(new RPackageVersion("9.9999-9999"), is(equalTo(new RPackageVersion("9.9999.9999"))));
    assertThat(new RPackageVersion("1.01"), is(equalTo(new RPackageVersion("1.1"))));
    assertThat(new RPackageVersion("1.01").hashCode(), is(equalTo(new RPackageVersion("1.1").hashCode())));
  }

  @Test
  public void distinguishesDifferentVersions() {
    assertThat(new RPackageVersion("0.2.2"), is(not(equalTo(new RPackageVersion("0.2-1")))));
    assertThat(new RPackageVersion("1.0"), is(not(equalTo(new RPackageVersion("1.0.0")))));
  }
}
