
package org.globalart.astrum.common.app;

import java.util.Properties;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;

/**
 * Tests for {@link ApplicationVersionSupport}.
 */
public class ApplicationVersionSupportTest
{
  private ApplicationVersionSupport underTest;

  private Properties props;

  @Before
  public void setUp() {
    props = new Properties();
    underTest = new ApplicationVersionSupport()
    {
      @Override
      public Properties getProperties() {
        return props;
      }

      @Override
      public String getEdition() {
        return "TEST";
      }
    };
  }

  /*
   * edition returns non-UNKNOWN
   */
  @Test
  public void testGetEdition() {
    assertThat(underTest.getEdition(), equalTo("TEST"));
  }

  @Test
  public void testGetVersion() {
    props.setProperty(ApplicationVersionSupport.VERSION, "123");
    assertThat(underTest.getVersion(), equalTo("123"));
  }

  @Test
  public void testGetVersion_missing() {
    assertThat(underTest.getVersion(), equalTo(ApplicationVersionSupport.UNKNOWN));
  }
}
