
package org.globalart.astrum.repository.view.matchers.token;

import java.text.CharacterIterator;
import java.text.StringCharacterIterator;
import java.util.Collections;
import java.util.List;

import org.junit.Test;

import static java.util.Arrays.asList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.repository.view.matchers.token.PatternParser.readFragment;

public class PatternParserFragmentReadingTest
{
  public static final String WILD = ".+";

  public static final List<Character> NONE = Collections.<Character>emptyList();

  @Test
  public void readLiteralFragment() {
    final String s = readFragment(it("abcdefg"), asList('-'), asList('-'), NONE);

    assertThat(s, is(equalTo("abcdefg")));
  }

  @Test
  public void readTerminatedFragment() {
    final String s = readFragment(it("abc-defg"), asList('-'), asList('\\'), NONE);
    assertThat(s, is(equalTo("abc")));
  }

  @Test
  public void readEscapedFragment() {
    final String s = readFragment(it("abc\\-d-efg"), asList('-'), asList('\\'), NONE);
    assertThat(s, is(equalTo("abc-d")));
  }

  @Test(expected = IllegalArgumentException.class)
  public void readDisallowedCharacter() {
    readFragment(it("s*fs"), NONE, NONE, asList('*'));
  }

  @Test(expected = IllegalArgumentException.class)
  public void escapingDoesNotProtectDisallowedCharacters() {

    readFragment(it("s\\*fs"), NONE, asList('\\'), asList('*'));
  }

  private CharacterIterator it(final String sdfs) {
    return new StringCharacterIterator(sdfs);
  }
}
