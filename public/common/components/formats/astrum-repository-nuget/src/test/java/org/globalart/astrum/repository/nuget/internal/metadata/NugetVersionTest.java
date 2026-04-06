package org.globalart.astrum.repository.nuget.internal.metadata;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class NugetVersionTest
{
  private static NugetVersion version(final String value) {
    return NugetVersion.parse(value).orElseThrow();
  }

  @Test
  public void normalizesLegacyAndFourPartVersions() {
    assertEquals("1.0.0", version("1.0").toNormalizedString());
    assertEquals("1.0.0", version("1.0.0.0").toNormalizedString());
    assertEquals("1.2.3.4", version("1.2.3.4").toNormalizedString());
    assertEquals("1.0.0", version("01.00").toNormalizedString());
    assertEquals("1.0.0-Beta.1", version("1.0.0-Beta.1+git.abc").toNormalizedString());
    assertEquals("1.0.0-Beta.1+git.abc", version("1.0.0-Beta.1+git.abc").toFullString());
    assertEquals("1.0.0-beta.1", version("1.0.0-Beta.1").toLowerString());
  }

  @Test
  public void rejectsInvalidVersions() {
    assertFalse(NugetVersion.parse("").isPresent());
    assertFalse(NugetVersion.parse("a.b").isPresent());
    assertFalse(NugetVersion.parse("1.0.0-").isPresent());
    assertFalse(NugetVersion.parse("1.2.3.4.5").isPresent());
  }

  @Test
  public void detectsPrereleaseAndSemVer2() {
    assertTrue(version("1.0.0-beta").isPrerelease());
    assertFalse(version("1.0.0-beta").isSemVer2());
    assertTrue(version("1.0.0-beta.1").isSemVer2());
    assertTrue(version("1.0.0+build").isSemVer2());
    assertFalse(version("1.0.0").isPrerelease());
  }

  @Test
  public void ordersBySemVerRules() {
    List<NugetVersion> versions = new ArrayList<>();
    for (String value : new String[]{"2.0.0", "1.0.0", "1.0.0-rc.10", "1.0.0-rc.2", "1.0.0-alpha", "1.0.0.1",
        "1.0.0-RC.1", "1.0.0-alpha.beta"}) {
      versions.add(version(value));
    }
    Collections.sort(versions);
    assertEquals("[1.0.0-alpha, 1.0.0-alpha.beta, 1.0.0-RC.1, 1.0.0-rc.2, 1.0.0-rc.10, 1.0.0, 1.0.0.1, 2.0.0]",
        versions.toString());
  }

  @Test
  public void comparesCaseInsensitivelyAndIgnoresMetadata() {
    assertEquals(version("1.0.0-BETA"), version("1.0.0-beta"));
    assertEquals(version("1.0.0+a"), version("1.0.0+b"));
    assertEquals(version("1.0"), version("1.0.0.0"));
  }
}
