
package org.globalart.astrum.repository.search.sql.query.syntax;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class RegexWildcardTermTest
{
  @Test
  public void testSingleQuestionMark() {
    RegexWildcardTerm term = new RegexWildcardTerm("com?ns-lang3");
    String regex = term.toRegexPattern();
    assertThat(regex, is("^com.ns-lang3$"));
  }

  @Test
  public void testTrailingQuestionMark() {
    RegexWildcardTerm term = new RegexWildcardTerm("commons?lang3?");
    String regex = term.toRegexPattern();
    assertThat(regex, is("^commons.lang3.$"));
  }

  @Test
  public void testMultipleAsteriskWithTrailingQuestionMark() {
    RegexWildcardTerm term = new RegexWildcardTerm("commons*lang?");
    String regex = term.toRegexPattern();
    assertThat(regex, is("^commons.*lang.$"));
  }

  @Test
  public void testLeadingQuestionMarkWithAsterisk() {
    RegexWildcardTerm term = new RegexWildcardTerm("commons?lang*");
    String regex = term.toRegexPattern();
    assertThat(regex, is("^commons.lang.*$"));
  }

  @Test
  public void testAsteriskInMiddle() {
    RegexWildcardTerm term = new RegexWildcardTerm("com*ns-lang3");
    String regex = term.toRegexPattern();
    assertThat(regex, is("^com.*ns-lang3$"));
  }

  @Test
  public void testMultipleAsterisks() {
    RegexWildcardTerm term = new RegexWildcardTerm("commons*lang3*");
    String regex = term.toRegexPattern();
    assertThat(regex, is("^commons.*lang3.*$"));
  }

  @Test
  public void testNoWildcards() {
    RegexWildcardTerm term = new RegexWildcardTerm("commons-lang3");
    String regex = term.toRegexPattern();
    assertThat(regex, is("^commons-lang3$"));
  }

  @Test
  public void testEscapeSequences() {
    RegexWildcardTerm term = new RegexWildcardTerm("commons\\*lang");
    String regex = term.toRegexPattern();
    assertThat(regex, is("^commons\\*lang$"));
  }

  @Test
  public void testEscapedQuestionMark() {
    RegexWildcardTerm term = new RegexWildcardTerm("commons\\?lang");
    String regex = term.toRegexPattern();
    assertThat(regex, is("^commons\\?lang$"));
  }

  @Test
  public void testRegexSpecialCharacters() {
    RegexWildcardTerm term = new RegexWildcardTerm("commons.lang");
    String regex = term.toRegexPattern();
    assertThat(regex, is("^commons\\.lang$"));
  }

  @Test
  public void testRegexSpecialCharacterBracket() {
    RegexWildcardTerm term = new RegexWildcardTerm("commons[lang]");
    String regex = term.toRegexPattern();
    assertThat(regex, is("^commons\\[lang\\]$"));
  }

  @Test
  public void testGetReturnsOriginalTerm() {
    RegexWildcardTerm term = new RegexWildcardTerm("commons*lang");
    assertThat(term.get(), is("commons*lang"));
  }

  @Test
  public void testNullTermThrowsNullPointerException() {
    org.junit.Assert.assertThrows(NullPointerException.class, () -> new RegexWildcardTerm(null));
  }

  @Test
  public void testEmptyPattern() {
    RegexWildcardTerm term = new RegexWildcardTerm("");
    String regex = term.toRegexPattern();
    assertThat(regex, is("^$"));
  }

  @Test
  public void testOnlyWildcards() {
    RegexWildcardTerm term = new RegexWildcardTerm("?");
    String regex = term.toRegexPattern();
    assertThat(regex, is("^.$"));
  }

  @Test
  public void testOnlyAsterisk() {
    RegexWildcardTerm term = new RegexWildcardTerm("*");
    String regex = term.toRegexPattern();
    assertThat(regex, is("^.*$"));
  }
}
