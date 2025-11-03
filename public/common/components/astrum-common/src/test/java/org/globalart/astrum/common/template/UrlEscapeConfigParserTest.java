
package org.globalart.astrum.common.template;

import java.util.Map;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.aMapWithSize;
import static org.hamcrest.Matchers.hasEntry;
import static org.hamcrest.Matchers.is;

public class UrlEscapeConfigParserTest
{
  @Test
  public void testParseRules_defaultFormat() {
    String config = "%:%25,::%3A,+:%2B, :%20,w:z";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(5));
    assertThat(rules, hasEntry("%", "%25"));
    assertThat(rules, hasEntry(":", "%3A"));
    assertThat(rules, hasEntry("+", "%2B"));
    assertThat(rules, hasEntry(" ", "%20"));
    assertThat(rules, hasEntry("w", "z"));
  }

  @Test
  public void testParseRules_emptyString() {
    String config = "";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(0));
  }

  @Test
  public void testParseRules_nullString() {
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(null);

    assertThat(rules, aMapWithSize(0));
  }

  @Test
  public void testParseRules_whitespaceOnly() {
    String config = "   ";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(0));
  }

  @Test
  public void testParseRules_singleRule() {
    String config = "+:%2B";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(1));
    assertThat(rules, hasEntry("+", "%2B"));
  }

  @Test
  public void testParseRules_multipleRules() {
    String config = "a:b,c:d,e:f";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(3));
    assertThat(rules, hasEntry("a", "b"));
    assertThat(rules, hasEntry("c", "d"));
    assertThat(rules, hasEntry("e", "f"));
  }

  @Test
  public void testParseRules_withColonInReplacement() {
    String config = "pat:%3A%3A";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(1));
    assertThat(rules, hasEntry("pat", "%3A%3A"));
  }

  @Test
  public void testParseRules_invalidFormatSkipped() {
    String config = "+:%2B,invalid,z:%20";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(2));
    assertThat(rules, hasEntry("+", "%2B"));
    assertThat(rules, hasEntry("z", "%20"));
  }

  @Test
  public void testParseRules_trailingComma() {
    String config = "+:%2B,x:%20,";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(2));
    assertThat(rules, hasEntry("+", "%2B"));
    assertThat(rules, hasEntry("x", "%20"));
  }

  @Test
  public void testParseRules_multipleCommas() {
    String config = "+:%2B,,x:%20";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(2));
    assertThat(rules, hasEntry("+", "%2B"));
    assertThat(rules, hasEntry("x", "%20"));
  }

  @Test
  public void testParseRules_emptyPattern() {
    String config = ":%2B";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(1));
    assertThat(rules, hasEntry("", "%2B"));
  }

  @Test
  public void testParseRules_emptyReplacement() {
    String config = "+:";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(1));
    assertThat(rules, hasEntry("+", ""));
  }

  @Test
  public void testParseRules_spaceAsPattern() {
    String config = " :%20,x:y";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(2));
    assertThat(rules, hasEntry(" ", "%20"));
    assertThat(rules, hasEntry("x", "y"));
  }

  @Test
  public void testParseRules_leadingAndTrailingSpacesInPattern() {
    String config = "a:  test  ";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(1));
    assertThat(rules, hasEntry("a", "  test  "));
  }

  @Test
  public void testParseRules_preservesOrder() {
    String config = "a:1,b:2,c:3";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(3));
    String[] keys = rules.keySet().toArray(new String[0]);
    assertThat(keys[0], is("a"));
    assertThat(keys[1], is("b"));
    assertThat(keys[2], is("c"));
  }

  @Test
  public void testParseRules_colonAsPattern() {
    String config = "::%3A";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(1));
    assertThat(rules, hasEntry(":", "%3A"));
  }

  @Test
  public void testParseRules_colonAsPatternWithOthers() {
    String config = "%:%25,::%3A,+:%2B";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(3));
    assertThat(rules, hasEntry("%", "%25"));
    assertThat(rules, hasEntry(":", "%3A"));
    assertThat(rules, hasEntry("+", "%2B"));
  }

  @Test
  public void testParseRules_configTooLong() {
    StringBuilder longConfig = new StringBuilder();
    for (int i = 0; i < 2001; i++) {
      longConfig.append("a");
    }
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(longConfig.toString());

    assertThat(rules, aMapWithSize(0));
  }

  @Test
  public void testParseRules_tooManyRules() {
    StringBuilder config = new StringBuilder();
    for (int i = 0; i < 25; i++) {
      if (i > 0) {
        config.append(",");
      }
      config.append("a").append(i).append(":b").append(i);
    }
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config.toString());

    assertThat(rules, aMapWithSize(20));
  }

  @Test
  public void testParseRules_patternTooLong() {
    StringBuilder longPattern = new StringBuilder();
    for (int i = 0; i < 51; i++) {
      longPattern.append("a");
    }
    String config = longPattern + ":%20";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(0));
  }

  @Test
  public void testParseRules_replacementTooLong() {
    StringBuilder longReplacement = new StringBuilder();
    for (int i = 0; i < 101; i++) {
      longReplacement.append("a");
    }
    String config = "x:" + longReplacement;
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(0));
  }

  @Test
  public void testParseRules_controlCharsInPattern() {
    String config = "test\n:value";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(0));
  }

  @Test
  public void testParseRules_controlCharsInReplacement() {
    String config = "test:val\rue";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(0));
  }

  @Test
  public void testParseRules_specialCharacters() {
    String config = "^:%5E,#:%23,&:%26";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(3));
    assertThat(rules, hasEntry("^", "%5E"));
    assertThat(rules, hasEntry("#", "%23"));
    assertThat(rules, hasEntry("&", "%26"));
  }

  @Test
  public void testParseRules_mixedValidAndInvalid() {
    String config = "a:b,invalidformat,c:d,toolongpattern" + "x".repeat(50) + ":e";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(2));
    assertThat(rules, hasEntry("a", "b"));
    assertThat(rules, hasEntry("c", "d"));
  }

  @Test
  public void testParseRules_unicodeCharacters() {
    String config = "\u00e9:\u00e8,\u4e2d:\u6587";
    Map<String, String> rules = UrlEscapeConfigParser.parseRules(config);

    assertThat(rules, aMapWithSize(2));
    assertThat(rules, hasEntry("\u00e9", "\u00e8"));
    assertThat(rules, hasEntry("\u4e2d", "\u6587"));
  }
}
