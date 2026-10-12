package org.globalart.astrum.repository.yum.internal.search;

import org.junit.Test;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class YumPreReleaseEvaluatorTest
{
  @Test
  public void recognizesRpmPreReleaseConventions() {
    assertTrue(YumPreReleaseEvaluator.isPreRelease("2.0-0.1.rc1.fc40"));
    assertTrue(YumPreReleaseEvaluator.isPreRelease("2.0~rc1-1.fc40"));
    assertTrue(YumPreReleaseEvaluator.isPreRelease("1:2.0~beta-1"));
    assertFalse(YumPreReleaseEvaluator.isPreRelease("2.0-1.fc40"));
    assertFalse(YumPreReleaseEvaluator.isPreRelease("1:2.0-10.el9"));
    assertFalse(YumPreReleaseEvaluator.isPreRelease(""));
  }
}
