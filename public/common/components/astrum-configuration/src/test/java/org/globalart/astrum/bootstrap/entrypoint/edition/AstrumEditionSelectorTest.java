
package org.globalart.astrum.bootstrap.entrypoint.edition;

import java.util.List;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.hamcrest.Matchers.nullValue;

public class AstrumEditionSelectorTest
{
  private AstrumEdition testEdition1;

  private AstrumEdition testEdition2;

  private AstrumEditionSelector underTest;

  @BeforeEach
  public void setup() {
    testEdition1 = new TestEdition1();
    testEdition2 = new TestEdition2();
    List<AstrumEdition> editions = List.of(testEdition1, testEdition2);
    underTest = new AstrumEditionSelector(editions);
  }

  @Test
  public void testGetCurrent() {
    assertThat(underTest.getCurrent(), not(nullValue()));
    assertThat(underTest.getCurrent(), is(testEdition1));
  }

  @Test
  public void testGetCurrent_swapEditionOrder() {
    List<AstrumEdition> editions = List.of(testEdition2, testEdition1);
    underTest = new AstrumEditionSelector(editions);

    // still same results
    assertThat(underTest.getCurrent(), not(nullValue()));
    assertThat(underTest.getCurrent(), is(testEdition1));
  }

  private static class TestEdition1
      extends AstrumEditionSupport
  {
    public TestEdition1() {
      super("test-1", "TEST 1", "TEST1", 100);
    }

    @Override
    public boolean isActive() {
      return true;
    }
  }

  private static class TestEdition2
      extends AstrumEditionSupport
  {
    public TestEdition2() {
      super("test-2", "TEST 2", "TEST2", 200);
    }

    @Override
    public boolean isActive() {
      return true;
    }
  }
}
