
package org.globalart.astrum.repository.search.sql.index;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

class SearchTokenizerTest
{
  @Test
  void testTsEscape() {
    assertThat(SearchTokenizer.tsEscape("asdf"), is("'asdf'"));
    assertThat(SearchTokenizer.tsEscape("as'df"), is("'as\\'df'"));
    assertThat(SearchTokenizer.tsEscape("AS'DF"), is("'as\\'df'"));
    assertThat(SearchTokenizer.tsEscape("as\\df"), is("'as\\\\df'"));
  }

  @Test
  void testStripLeadingSeparators() {
    assertThat(SearchTokenizer.stripLeadingSeparators(""), is(""));
    assertThat(SearchTokenizer.stripLeadingSeparators("-"), is(""));
    assertThat(SearchTokenizer.stripLeadingSeparators("-asdf"), is("asdf"));
    assertThat(SearchTokenizer.stripLeadingSeparators("/-\\._ asdf"), is("asdf"));
  }

  @Test
  void testStripTrailingSeparators() {
    assertThat(SearchTokenizer.stripTrailingSeparators(""), is(""));
    assertThat(SearchTokenizer.stripTrailingSeparators("-"), is(""));
    assertThat(SearchTokenizer.stripTrailingSeparators("asdf-"), is("asdf"));
    assertThat(SearchTokenizer.stripTrailingSeparators("EUCLID/"), is("EUCLID"));
    assertThat(SearchTokenizer.stripTrailingSeparators("EUCLID"), is("EUCLID"));
    assertThat(SearchTokenizer.stripTrailingSeparators("/EUCLID/"), is("/EUCLID"));
    assertThat(SearchTokenizer.stripTrailingSeparators("term."), is("term"));
    assertThat(SearchTokenizer.stripTrailingSeparators("asdf /-\\._"), is("asdf"));
    assertThat(SearchTokenizer.stripTrailingSeparators("astrum,"), is("astrum"));
  }

  @ParameterizedTest
  @CsvSource({
      "package_id, true", "package.id, true", "package-id, true",
      "package\\id, true", "package/id, true", "package id, true", "'package,id', true",
      "packageid, false", "package*id, false", "package#id, false"
  })
  void testTokenizerRegex(String expression, boolean expectedValue) {
    assertThat(SearchTokenizer.TOKENIZER.matcher(expression).find(), is(expectedValue));
  }
}
