
package org.globalart.astrum.security.authz;

import java.util.ArrayList;
import java.util.List;
import java.util.Set;

import com.google.common.collect.ImmutableSet;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

import static java.util.Arrays.asList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

@RunWith(MockitoJUnitRunner.Silent.class)
public class WildcardPermission2Test
{
  private static final List<String> SUB_PARTS = asList("subPart1", "subPart2");

  private static final List<String> MULTIPLE_ACTIONS = asList("actionNumber1", "actionNumber2");

  private static final List<String> SINGLE_ACTION = asList("singleAction");

  private static final boolean CASE_SENSITIVE = true;

  @Test
  public void testSetPartsNotCaseSensitive() {
    List<Set<String>> expectedParts = new ArrayList<>();
    expectedParts.add(ImmutableSet.of("subpart1"));
    expectedParts.add(ImmutableSet.of("subpart2"));
    expectedParts.add(ImmutableSet.of("actionnumber1", "actionnumber2"));

    verifyParts(SUB_PARTS, MULTIPLE_ACTIONS, !CASE_SENSITIVE, expectedParts);
  }

  @Test
  public void testSetPartsCaseSensitive() {
    List<Set<String>> expectedParts = new ArrayList<>();
    expectedParts.add(ImmutableSet.of("subPart1"));
    expectedParts.add(ImmutableSet.of("subPart2"));
    expectedParts.add(ImmutableSet.of("actionNumber1", "actionNumber2"));

    verifyParts(SUB_PARTS, MULTIPLE_ACTIONS, CASE_SENSITIVE, expectedParts);
  }

  @Test
  public void setPartsSingleAction() {
    List<Set<String>> expectedParts = new ArrayList<>();
    expectedParts.add(ImmutableSet.of("subpart1"));
    expectedParts.add(ImmutableSet.of("subpart2"));
    expectedParts.add(ImmutableSet.of("singleaction"));

    verifyParts(SUB_PARTS, SINGLE_ACTION, !CASE_SENSITIVE, expectedParts);
  }

  private static void verifyParts(
      final List<String> subParts,
      final List<String> actions,
      final boolean caseSensitive,
      final List<Set<String>> expectedParts)
  {
    WildcardPermission2 underTest = new WildcardPermission2();
    underTest.setParts(subParts, actions, caseSensitive);
    assertThat(underTest.getParts(), is(expectedParts));
  }
}
