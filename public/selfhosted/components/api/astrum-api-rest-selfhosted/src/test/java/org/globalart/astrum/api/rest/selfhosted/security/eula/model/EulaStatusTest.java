
package org.globalart.astrum.api.rest.selfhosted.security.eula.model;

import org.junit.Test;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class EulaStatusTest
{
  private final EulaStatus underTest = new EulaStatus();

  @Test
  public void testValidDisclaimer() {
    underTest.setDisclaimer(
        "Use of GlobalArt Astrum Repository - Community Edition is governed by the End User License Agreement at https://links.globalart.dev/products/astrum/ce-eula. By returning the value from ‘accepted:false’ to ‘accepted:true’, you acknowledge that you have read and agree to the End User License Agreement at https://links.globalart.dev/products/astrum/ce-eula.");
    assertTrue(underTest.hasExpectedDisclaimer());
  }

  @Test
  public void testInvalidDisclaimer() {
    underTest.setDisclaimer("invalid");
    assertFalse(underTest.hasExpectedDisclaimer());
  }

  @Test
  public void testNullDisclaimer() {
    underTest.setDisclaimer(null);
    assertFalse(underTest.hasExpectedDisclaimer());
  }

  @Test
  public void testEmptyDisclaimer() {
    underTest.setDisclaimer("");
    assertFalse(underTest.hasExpectedDisclaimer());
  }
}
