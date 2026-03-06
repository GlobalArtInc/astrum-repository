
package org.globalart.astrum.cleanup.storage;

import java.util.Arrays;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasItemInArray;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.sameInstance;

/**
 * Tests for {@link SortType} and {@link CleanupPolicyReleaseType}.
 */
public class CleanupPolicyEnumsTest
{
  @Test
  public void testSortTypeValues() {
    assertThat(SortType.VERSION.value, is("version"));
    assertThat(SortType.DATE.value, is("date"));
  }

  @Test
  public void testSortTypeValuesArray() {
    SortType[] values = SortType.values();

    assertThat(values.length, is(2));
    assertThat(values, hasItemInArray(SortType.VERSION));
    assertThat(values, hasItemInArray(SortType.DATE));
  }

  @Test
  public void testSortTypeValueOf() {
    assertThat(SortType.valueOf("VERSION"), is(sameInstance(SortType.VERSION)));
    assertThat(SortType.valueOf("DATE"), is(sameInstance(SortType.DATE)));
  }

  @Test(expected = IllegalArgumentException.class)
  public void testSortTypeValueOfUnknownThrows() {
    SortType.valueOf("UNKNOWN");
  }

  @Test
  public void testCleanupPolicyReleaseTypeValuesArray() {
    CleanupPolicyReleaseType[] values = CleanupPolicyReleaseType.values();

    assertThat(values.length, is(2));
    assertThat(values, hasItemInArray(CleanupPolicyReleaseType.RELEASES));
    assertThat(values, hasItemInArray(CleanupPolicyReleaseType.PRERELEASES));
  }

  @Test
  public void testCleanupPolicyReleaseTypeValuesOrder() {
    // RELEASES is declared first, PRERELEASES second
    assertThat(Arrays.asList(CleanupPolicyReleaseType.values()),
        is(Arrays.asList(CleanupPolicyReleaseType.RELEASES, CleanupPolicyReleaseType.PRERELEASES)));
  }

  @Test
  public void testCleanupPolicyReleaseTypeValueOf() {
    assertThat(CleanupPolicyReleaseType.valueOf("RELEASES"), is(sameInstance(CleanupPolicyReleaseType.RELEASES)));
    assertThat(CleanupPolicyReleaseType.valueOf("PRERELEASES"), is(sameInstance(CleanupPolicyReleaseType.PRERELEASES)));
  }

  @Test(expected = IllegalArgumentException.class)
  public void testCleanupPolicyReleaseTypeValueOfUnknownThrows() {
    CleanupPolicyReleaseType.valueOf("UNKNOWN");
  }
}
