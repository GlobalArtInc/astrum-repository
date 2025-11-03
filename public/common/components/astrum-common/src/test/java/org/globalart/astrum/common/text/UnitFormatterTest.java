
package org.globalart.astrum.common.text;

import org.junit.Test;

import static java.lang.Math.pow;
import static java.lang.Math.round;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class UnitFormatterTest

{
  @Test
  public void testFormatStorage_units() {
    assertThat(UnitFormatter.formatStorage(round(pow(10, 18))), is("1.00 EB"));
    assertThat(UnitFormatter.formatStorage(round(pow(10, 15))), is("1.00 PB"));
    assertThat(UnitFormatter.formatStorage(round(pow(10, 12))), is("1.00 TB"));
    assertThat(UnitFormatter.formatStorage(round(pow(10, 9))), is("1.00 GB"));
    assertThat(UnitFormatter.formatStorage(round(pow(10, 6))), is("1.00 MB"));
    assertThat(UnitFormatter.formatStorage(round(pow(10, 3))), is("1.00 KB"));
    assertThat(UnitFormatter.formatStorage(round(pow(10, 0))), is("1.00 B"));
    assertThat(UnitFormatter.formatStorage(0), is("0.00 B"));
    assertThat(UnitFormatter.formatStorage(-1 * round(pow(10, 18))), is("-1.00 EB"));
    assertThat(UnitFormatter.formatStorage(-1 * round(pow(10, 15))), is("-1.00 PB"));
    assertThat(UnitFormatter.formatStorage(-1 * round(pow(10, 12))), is("-1.00 TB"));
    assertThat(UnitFormatter.formatStorage(-1 * round(pow(10, 9))), is("-1.00 GB"));
    assertThat(UnitFormatter.formatStorage(-1 * round(pow(10, 6))), is("-1.00 MB"));
    assertThat(UnitFormatter.formatStorage(-1 * round(pow(10, 3))), is("-1.00 KB"));
    assertThat(UnitFormatter.formatStorage(-1 * round(pow(10, 0))), is("-1.00 B"));
  }

  @Test
  public void testFormatStorage_rounding() {
    assertThat(UnitFormatter.formatStorage(round(3.7 * pow(10, 18))), is("3.70 EB"));
    assertThat(UnitFormatter.formatStorage(round(3.4 * pow(10, 18))), is("3.40 EB"));
    assertThat(UnitFormatter.formatStorage(round(2.999 * pow(10, 18))), is("3.00 EB"));
    assertThat(UnitFormatter.formatStorage(round(2.444 * pow(10, 18))), is("2.44 EB"));
  }
}
