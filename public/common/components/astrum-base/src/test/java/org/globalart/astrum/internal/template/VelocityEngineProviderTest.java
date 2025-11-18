
package org.globalart.astrum.internal.template;

import org.apache.velocity.app.VelocityEngine;
import org.junit.Test;

import static java.lang.Integer.parseInt;
import static org.apache.velocity.runtime.RuntimeConstants.PARSER_POOL_SIZE;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class VelocityEngineProviderTest
{
  @Test
  public void testParserPoolSizeIsSet() {
    final int velocityParserPoolSize = 30;
    VelocityEngineProvider underTest = new VelocityEngineProvider(velocityParserPoolSize);

    final VelocityEngine velocityEngine = underTest.getObject();

    assertThat(parseInt(velocityEngine.getProperty(PARSER_POOL_SIZE).toString()), is(velocityParserPoolSize));
  }
}
