
package org.globalart.astrum.repository.search.sql.query.security;

import org.globalart.astrum.repository.search.sql.query.syntax.ExactTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.NullTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.Operand;
import org.globalart.astrum.repository.search.sql.query.syntax.WildcardTerm;
import org.globalart.astrum.selector.CselToSql;
import org.globalart.astrum.selector.ParserVisitorSupport;

import org.apache.commons.jexl3.JexlException;
import org.apache.commons.jexl3.parser.ASTAndNode;
import org.apache.commons.jexl3.parser.ASTEQNode;
import org.apache.commons.jexl3.parser.ASTERNode;
import org.apache.commons.jexl3.parser.ASTIdentifier;
import org.apache.commons.jexl3.parser.ASTIdentifierAccess;
import org.apache.commons.jexl3.parser.ASTJexlScript;
import org.apache.commons.jexl3.parser.ASTNENode;
import org.apache.commons.jexl3.parser.ASTOrNode;
import org.apache.commons.jexl3.parser.ASTReference;
import org.apache.commons.jexl3.parser.ASTReferenceExpression;
import org.apache.commons.jexl3.parser.ASTSWNode;
import org.apache.commons.jexl3.parser.ASTStringLiteral;
import org.apache.commons.jexl3.parser.JexlNode;

import static org.apache.commons.lang3.StringUtils.removeEnd;
import static org.apache.commons.lang3.StringUtils.removeStart;

/**
 * Walks the script,transforming CSEL into {@link Expression} for use by SQL Search queries.
 * This abstract class must be extended by database-specific implementations.
 */
public abstract class CselToExpression
    extends ParserVisitorSupport
    implements CselToSql<SelectorExpressionBuilder>
{
  private static final String EXPECTED_STRING_LITERAL = "Expected string literal";

  /**
   * Get the token start regex pattern for database-specific path matching.
   */
  protected abstract String getTokenStartRegex();

  /**
   * Get the token end regex pattern for database-specific path matching.
   */
  protected abstract String getTokenEndRegex();

  @Override
  public void transformCselToSql(final ASTJexlScript script, final SelectorExpressionBuilder builder) {
    script.childrenAccept(this, builder);
  }

  @Override
  protected Object doVisit(final JexlNode node, final Object data) {
    throw new JexlException(node,
        "Expression not supported in CSEL selector, failing node is " + node.jexlInfo().toString());
  }

  /**
   * Transform `a || b` into `a or b`
   */
  @Override
  protected Object visit(final ASTOrNode node, final Object data) {
    return transformOperator(node, Operand.OR, (SelectorExpressionBuilder) data);
  }

  /**
   * Transform `a && b` into `a and b`
   */
  @Override
  protected Object visit(final ASTAndNode node, final Object data) {
    return transformOperator(node, Operand.AND, (SelectorExpressionBuilder) data);
  }

  /**
   * Transform `a == b` into `a @@ TO_TSQUERY('simple', b)`
   */
  @Override
  protected Object visit(final ASTEQNode node, final Object data) {
    transformOperator(node, Operand.EQ, (SelectorExpressionBuilder) data);
    return data;
  }

  /**
   * Transform `a =~ b` into `a ~ b`
   */
  @Override
  protected Object visit(final ASTERNode node, final Object data) {
    return transformMatchesOperator(node, (SelectorExpressionBuilder) data);
  }

  /**
   * Transform `a != b` into `(a is null or a @@ !!to_tsquery('simple', b)`
   */
  @Override
  protected Object visit(final ASTNENode node, final Object data) {
    JexlNode leftChild = node.jjtGetChild(LEFT);
    JexlNode rightChild = node.jjtGetChild(RIGHT);
    if (rightChild instanceof ASTStringLiteral) {
      transformNotEqualsOperator(leftChild, (ASTStringLiteral) rightChild, (SelectorExpressionBuilder) data);
    }
    else if (leftChild instanceof ASTStringLiteral) {
      transformNotEqualsOperator(rightChild, (ASTStringLiteral) leftChild, (SelectorExpressionBuilder) data);
    }
    else {
      throw new JexlException(node, EXPECTED_STRING_LITERAL);
    }
    return data;
  }

  /**
   * Transform `a =^ "something"` into `a @@ TO_TSQUERY('simple', something:*)"`
   */
  @Override
  protected Object visit(final ASTSWNode node, final Object data) {
    JexlNode leftChild = node.jjtGetChild(LEFT);
    JexlNode rightChild = node.jjtGetChild(RIGHT);
    SelectorExpressionBuilder builder = (SelectorExpressionBuilder) data;
    if (rightChild instanceof ASTStringLiteral) {
      transformStartsWithOperator(leftChild, (ASTStringLiteral) rightChild, builder);
    }
    else if (leftChild instanceof ASTStringLiteral) {
      transformStartsWithOperator(rightChild, (ASTStringLiteral) leftChild, builder);
    }
    else {
      throw new JexlException(node, EXPECTED_STRING_LITERAL);
    }
    return data;
  }

  /**
   * Apply `( expression )`
   */
  @Override
  protected Object visit(final ASTReferenceExpression node, final Object data) {
    ((SelectorExpressionBuilder) data).appendExpression(() -> node.childrenAccept(this, data));
    return data;
  }

  /**
   * Transform identifiers into asset fields.
   */
  @Override
  protected Object visit(final ASTIdentifier node, final Object data) {
    ((SelectorExpressionBuilder) data).appendField(node.getName());
    return data;
  }

  /**
   * Store string literals as parameters and specify as <code>to_tsquery()</code> argument.
   */
  @Override
  protected Object visit(final ASTStringLiteral node, final Object data) {
    ((SelectorExpressionBuilder) data).appendTerm(new ExactTerm(node.getLiteral()));
    return data;
  }

  /**
   * Transform dotted references into format-specific attributes.
   */
  @Override
  protected Object visit(final ASTReference node, final Object data) {
    ASTIdentifierAccess subRef = (ASTIdentifierAccess) node.jjtGetChild(RIGHT);
    ((SelectorExpressionBuilder) data).appendField(subRef.getName());
    return data;
  }

  protected SelectorExpressionBuilder transformOperator(
      final JexlNode node,
      final Operand operator,
      final SelectorExpressionBuilder builder)
  {
    JexlNode leftChild = node.jjtGetChild(LEFT);
    JexlNode rightChild = node.jjtGetChild(RIGHT);
    leftChild.jjtAccept(this, builder);
    builder.appendOperand(operator);
    rightChild.jjtAccept(this, builder);
    return builder;
  }

  private SelectorExpressionBuilder transformStartsWithOperator(
      final JexlNode node,
      final ASTStringLiteral literal,
      final SelectorExpressionBuilder builder)
  {
    node.jjtAccept(this, builder);
    builder.appendOperand(Operand.EQ);
    builder.appendTerm(new WildcardTerm(literal.getLiteral(), false));
    return builder;
  }

  /**
   * Builds the regex expression for a regex selector.
   * The database column that it checks is a VARCHAR containing one or more paths.
   *
   * Format varies by database:
   * - PostgresSQL: {'/foo/bar'} {'/baz/qux'}
   * - H2: ["/foo/bar","/baz/qux"]
   *
   */
  protected SelectorExpressionBuilder transformMatchesOperator(
      final JexlNode node,
      final SelectorExpressionBuilder builder)
  {
    JexlNode leftChild = node.jjtGetChild(LEFT);
    JexlNode rightChild = node.jjtGetChild(RIGHT);

    builder.appendField(((ASTIdentifier) leftChild).getName());

    builder.appendOperand(Operand.REGEX);
    if (rightChild instanceof ASTStringLiteral) {
      String pattern = ((ASTStringLiteral) rightChild).getLiteral();

      if (pattern.charAt(0) != '^') {
        if (pattern.charAt(pattern.length() - 1) == '$') {
          pattern = removeEnd(pattern, "$");
        }
        pattern = String.format(getTokenStartRegex() + "(%s)" + getTokenEndRegex(), pattern); // match entire string
      }
      else {
        // Handle anchored patterns properly to avoid corrupting nested parentheses
        String innerPattern = removeStart(pattern, "^");
        boolean endsWithDollar = innerPattern.endsWith("$");

        if (endsWithDollar) {
          innerPattern = removeEnd(innerPattern, "$");
          pattern = getTokenStartRegex() + innerPattern + getTokenEndRegex();
        }
        else {
          pattern = getTokenStartRegex() + innerPattern;
        }
      }

      builder.appendTerm(new ExactTerm(pattern));
    }
    else {
      throw new JexlException(node, EXPECTED_STRING_LITERAL);
    }
    return builder;
  }

  private SelectorExpressionBuilder transformNotEqualsOperator(
      final JexlNode node,
      final ASTStringLiteral literal,
      final SelectorExpressionBuilder builder)
  {
    builder.appendExpression(() -> {
      node.jjtAccept(this, builder);
      builder.appendOperand(Operand.EQ);
      builder.appendTerm(NullTerm.INSTANCE);
      builder.appendOperand(Operand.OR);
      node.jjtAccept(this, builder);

      builder.appendOperand(Operand.NOT_EQ);
      literal.jjtAccept(this, builder);
    });
    return builder;
  }
}
