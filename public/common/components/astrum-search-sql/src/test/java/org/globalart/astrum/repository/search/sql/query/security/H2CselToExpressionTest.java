
package org.globalart.astrum.repository.search.sql.query.security;

import org.globalart.astrum.repository.rest.sql.SearchField;
import org.globalart.astrum.repository.search.sql.query.syntax.ExactTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.Operand;
import org.globalart.astrum.repository.search.sql.query.syntax.SqlPredicate;

import org.apache.commons.jexl3.parser.ASTJexlScript;
import org.junit.jupiter.api.Test;

class H2CselToExpressionTest
    extends CselToExpressionTest
{
  @Override
  protected CselToExpression createUnderTest() {
    return new H2CselToExpression();
  }

  @Test
  public void regexpTest() {
    ASTJexlScript script = jexlEngine.parseExpression("a =~ \"woof\"");

    script.childrenAccept(underTest, builder);

    // "paths_alias ~ :param_0"
    assertPredicate((SqlPredicate) builder.build(), SearchField.FORMAT_FIELD_1, Operand.REGEX,
        new ExactTerm("(\\[\"|,\")(woof)(\",|\"\\])"));

    reset();

    script = jexlEngine.parseExpression("a =~ \"woof$\""); // match always starts from start of path
    script.childrenAccept(underTest, builder);

    assertPredicate((SqlPredicate) builder.build(), SearchField.FORMAT_FIELD_1, Operand.REGEX,
        new ExactTerm("(\\[\"|,\")(woof)(\",|\"\\])"));

    reset();

    script = jexlEngine.parseExpression("a =~ \"^woof$\"");
    script.childrenAccept(underTest, builder);

    assertPredicate((SqlPredicate) builder.build(), SearchField.FORMAT_FIELD_1, Operand.REGEX,
        new ExactTerm("(\\[\"|,\")woof(\",|\"\\])"));

    reset();

    script = jexlEngine.parseExpression("a =~ \"^woof\"");
    script.childrenAccept(underTest, builder);

    assertPredicate((SqlPredicate) builder.build(), SearchField.FORMAT_FIELD_1, Operand.REGEX,
        new ExactTerm("(\\[\"|,\")woof"));

    reset();

    script = jexlEngine.parseExpression("a =~ \"^/woof|/woof/foo\"");

    script.childrenAccept(underTest, builder);

    assertPredicate((SqlPredicate) builder.build(), SearchField.FORMAT_FIELD_1, Operand.REGEX,
        new ExactTerm("(\\[\"|,\")/woof|/woof/foo"));
  }

  @Test
  public void regexpTestWithNestedParentheses() {
    // Test simple negative lookahead - should work
    ASTJexlScript script = jexlEngine.parseExpression("a =~ \"^(?!/test).*$\"");
    script.childrenAccept(underTest, builder);

    assertPredicate((SqlPredicate) builder.build(), SearchField.FORMAT_FIELD_1, Operand.REGEX,
        new ExactTerm("(\\[\"|,\")(?!/test).*(\",|\"\\])"));

    reset();

    // Test complex nested parentheses - the original buggy case
    script = jexlEngine.parseExpression("a =~ \"^(?!(/com/customer/(nested))).*$\"");
    script.childrenAccept(underTest, builder);

    // Should preserve the nested parentheses structure
    assertPredicate((SqlPredicate) builder.build(), SearchField.FORMAT_FIELD_1, Operand.REGEX,
        new ExactTerm("(\\[\"|,\")(?!(/com/customer/(nested))).*(\",|\"\\])"));

    reset();

    // Test nested parentheses without anchors - should also work
    script = jexlEngine.parseExpression("a =~ \"(?!(/com/customer/(nested))).*\"");
    script.childrenAccept(underTest, builder);

    assertPredicate((SqlPredicate) builder.build(), SearchField.FORMAT_FIELD_1, Operand.REGEX,
        new ExactTerm("(\\[\"|,\")((?!(/com/customer/(nested))).*)(\",|\"\\])"));

    reset();

    // Test multiple levels of nesting
    script = jexlEngine.parseExpression("a =~ \"^(?=(.*test.*))(?!(.*debug.*)).*$\"");
    script.childrenAccept(underTest, builder);

    assertPredicate((SqlPredicate) builder.build(), SearchField.FORMAT_FIELD_1, Operand.REGEX,
        new ExactTerm("(\\[\"|,\")(?=(.*test.*))(?!(.*debug.*)).*(\",|\"\\])"));

    reset();

    // Test nested alternation groups
    script = jexlEngine.parseExpression("a =~ \"^/(com|org|(net|io))/.*$\"");
    script.childrenAccept(underTest, builder);

    assertPredicate((SqlPredicate) builder.build(), SearchField.FORMAT_FIELD_1, Operand.REGEX,
        new ExactTerm("(\\[\"|,\")/(com|org|(net|io))/.*(\",|\"\\])"));

    reset();

    // Test anchored pattern without end anchor - should only add start delimiter
    script = jexlEngine.parseExpression("a =~ \"^(?!(/com/test)).*\"");
    script.childrenAccept(underTest, builder);

    assertPredicate((SqlPredicate) builder.build(), SearchField.FORMAT_FIELD_1, Operand.REGEX,
        new ExactTerm("(\\[\"|,\")(?!(/com/test)).*"));
  }
}
