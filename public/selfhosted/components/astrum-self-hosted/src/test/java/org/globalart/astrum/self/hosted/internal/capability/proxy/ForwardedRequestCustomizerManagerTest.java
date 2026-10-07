
package org.globalart.astrum.self.hosted.internal.capability.proxy;

import org.globalart.astrum.bootstrap.jetty.CustomForwardedRequestCustomizer;

import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * Tests for {@link ForwardedRequestCustomizerManager}.
 */
@ExtendWith(MockitoExtension.class)
class ForwardedRequestCustomizerManagerTest
{
  private ForwardedRequestCustomizerManager underTest;

  @BeforeEach
  void setUp() {
    underTest = new ForwardedRequestCustomizerManager();
  }

  @AfterEach
  void teardown() {
    CustomForwardedRequestCustomizer.clearInstance();
  }

  @Test
  void testSetEnabled_WhenCustomizerPresent() {
    CustomForwardedRequestCustomizer customizer = new CustomForwardedRequestCustomizer();

    underTest.setEnabled(false);

    assertThat(customizer.isEnabled(), is(false));
    assertThat(underTest.isEnabled(), is(false));
  }

  @Test
  void testSetEnabled_TogglesValue() {
    // This triggers creation
    new CustomForwardedRequestCustomizer();

    underTest.setEnabled(false);
    assertThat(underTest.isEnabled(), is(false));

    underTest.setEnabled(true);
    assertThat(underTest.isEnabled(), is(true));
  }
}
