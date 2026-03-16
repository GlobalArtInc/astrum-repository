package org.globalart.astrum.repository.search.sql.query.postgres;

import java.util.Locale;
import java.util.Map;

import org.globalart.astrum.repository.rest.sql.SearchField;
import org.globalart.astrum.repository.search.sql.ExpressionGroup;
import org.globalart.astrum.repository.search.sql.query.SqlSearchQueryCondition;
import org.globalart.astrum.repository.search.sql.query.SqlSearchQueryConditionGroup;
import org.globalart.astrum.repository.search.sql.query.postgres.PostgresSearchConditionBuilder.ConditionType;
import org.globalart.astrum.repository.search.sql.query.syntax.BooleanTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.ExactTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.Expression;
import org.globalart.astrum.repository.search.sql.query.syntax.LenientTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.NullTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.Operand;
import org.globalart.astrum.repository.search.sql.query.syntax.RegexWildcardTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.SqlClause;
import org.globalart.astrum.repository.search.sql.query.syntax.SqlPredicate;
import org.globalart.astrum.repository.search.sql.query.syntax.TermCollection;
import org.globalart.astrum.repository.search.sql.query.syntax.WildcardTerm;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.junit.jupiter.api.Assertions.assertThrows;

class PostgresFulltextSearchConditionBuilderTest
{
  @Test
  void testBooleanTerm() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.PRERELEASE, new BooleanTerm(true)));

    assertThat(result, is(new SqlSearchQueryCondition("cs.prerelease = #{filterParams.cs_prerelease0}",
        Map.of("cs_prerelease0", true))));
  }

  @Test
  void testExactTermOnTsvectorColumnIsLowercasedLexeme() {
    SqlSearchQueryCondition result = build(new SqlPredicate(Operand.EQ, SearchField.KEYWORDS, new ExactTerm("Tomcat")));

    assertThat(result.getSqlFilter(), is("cs.keywords @@ #{filterParams.cs_keywords0}::tsquery"));
    assertThat(result.getParameters(), is(Map.of("cs_keywords0", "'tomcat'")));
  }

  @Test
  void testWildcardTermOnTsvectorColumnUsesPrefixLexemes() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.KEYWORDS, new WildcardTerm("Org.Apache")));

    assertThat(result.getSqlFilter(), is("cs.keywords @@ #{filterParams.cs_keywords0}::tsquery"));
    assertThat(result.getParameters().get("cs_keywords0"), is("'org.apache':* | 'org' <-> 'apache':*"));
  }

  @Test
  void testWildcardTermWithTrailingAsteriskAndSeparators() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.KEYWORDS, new WildcardTerm("-provider.*")));

    assertThat(result.getParameters().get("cs_keywords0"), is("'-provider.':* | 'provider':*"));
  }

  @Test
  void testWildcardPathKeepsLeadingSeparatorInWholeLexeme() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.PATHS, new WildcardTerm("/org/test/file_1.jar")));

    assertThat(result.getParameters().get("cs_tsvector_paths0"),
        is("'/org/test/file_1.jar':* | 'org/test/file_1.jar':* | 'org' <-> 'test' <-> 'file' <-> '1' <-> 'jar':*"));
  }

  @Test
  void testLenientTerm() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.KEYWORDS, new LenientTerm("search")));

    assertThat(result.getSqlFilter(), is("cs.keywords @@ #{filterParams.cs_keywords0}::tsquery"));
    assertThat(result.getParameters().get("cs_keywords0"), is("'search'"));
  }

  @Test
  void testLenientTermIsTokenized() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.FORMAT_FIELD_1, new LenientTerm("Foo.Bar-baz")));

    assertThat(result.getSqlFilter(), is("cs.format_field_values_1 @@ #{filterParams.cs_format_field_values_10}::tsquery"));
    assertThat(result.getParameters().get("cs_format_field_values_10"),
        is("'foo.bar-baz' | 'foo' <-> 'bar' <-> 'baz'"));
  }

  @Test
  void testMultipleTermsWithAndLogic() {
    SqlSearchQueryCondition result = build(new SqlPredicate(Operand.EQ, SearchField.KEYWORDS,
        TermCollection.create(new ExactTerm("maven"), new ExactTerm("plugin"))));

    assertThat(result.getSqlFilter(), is("cs.keywords @@ #{filterParams.cs_keywords0}::tsquery"));
    assertThat(result.getParameters().get("cs_keywords0"), is("('maven') & ('plugin')"));
  }

  @Test
  void testMultipleTermsWithOrLogic() {
    SqlSearchQueryCondition result = build(new SqlPredicate(Operand.ANY, SearchField.KEYWORDS,
        TermCollection.create(new ExactTerm("junit"), new WildcardTerm("mock"))));

    assertThat(result.getParameters().get("cs_keywords0"), is("('junit') | ('mock':*)"));
  }

  @Test
  void testInOperatorOnTsvectorColumn() {
    SqlSearchQueryCondition result = build(new SqlPredicate(Operand.IN, SearchField.MD5,
        TermCollection.create(new ExactTerm("ABC"), new ExactTerm("def"))));

    assertThat(result.getSqlFilter(), is("cs.md5 @@ #{filterParams.cs_md50}::tsquery"));
    assertThat(result.getParameters().get("cs_md50"), is("('abc') | ('def')"));
  }

  @Test
  void testWildcardWithLeadingSeparators() {
    SqlSearchQueryCondition result = build(new SqlPredicate(Operand.EQ, SearchField.KEYWORDS,
        TermCollection.create(new WildcardTerm("maven"), new WildcardTerm("-provider"))));

    assertThat(result.getParameters().get("cs_keywords0"), is("('maven':*) & ('-provider':* | 'provider':*)"));
  }

  @Test
  void testQuotesAndBackslashesAreEscapedInsideLexemes() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.KEYWORDS, new ExactTerm("It's a\\b")));

    assertThat(result.getParameters().get("cs_keywords0"), is("'it\\'s a\\\\b'"));
  }

  @Test
  void testUserInputNeverReachesSqlText() {
    String hostile = "x'); DROP TABLE search_components; --";
    SqlSearchQueryCondition result = build(SqlClause.create(Operand.AND,
        new SqlPredicate(Operand.EQ, SearchField.KEYWORDS, new WildcardTerm(hostile)),
        new SqlPredicate(Operand.EQ, SearchField.NAME, new WildcardTerm(hostile)),
        new SqlPredicate(Operand.EQ, SearchField.REPOSITORY_NAME, new LenientTerm(hostile)),
        new SqlPredicate(Operand.EQ, SearchField.NAMESPACE, new ExactTerm(hostile))));

    assertThat(result.getSqlFilter(), not(containsString("DROP")));
    assertThat(result.getSqlFilter(), not(containsString("x'")));
    assertThat(result.getParameters().values().stream().allMatch(v -> v.toString().toLowerCase(Locale.ROOT)
        .contains("drop table")), is(true));
  }

  @Test
  void testBlankStringFiltering() {
    SqlSearchQueryCondition result = build(new SqlPredicate(Operand.EQ, SearchField.KEYWORDS,
        TermCollection.create(new ExactTerm("   "), new ExactTerm(""))));

    assertThat(result.getSqlFilter(), is(""));
    assertThat(result.getParameters().isEmpty(), is(true));
  }

  @Test
  void testBlankStringFilteringWithValidTerm() {
    SqlSearchQueryCondition result = build(new SqlPredicate(Operand.EQ, SearchField.KEYWORDS,
        TermCollection.create(new ExactTerm("   "), new ExactTerm("valid"))));

    assertThat(result.getSqlFilter(), is("cs.keywords @@ #{filterParams.cs_keywords0}::tsquery"));
    assertThat(result.getParameters().get("cs_keywords0"), is("'valid'"));
  }

  @Test
  void testSeparatorOnlyWildcardIsDropped() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.KEYWORDS, new WildcardTerm("-./")));

    assertThat(result.getSqlFilter(), is(""));
  }

  @Test
  void testSingleExactTermOnPlainColumnUsesEquality() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.NAME, new ExactTerm("Component-Name")));

    assertThat(result.getSqlFilter(), is("cs.search_component_name = #{filterParams.cs_search_component_name0}"));
    assertThat(result.getParameters().get("cs_search_component_name0"), is("Component-Name"));
  }

  @Test
  void testComponentVersionFilterUsesPlainColumnsNotTsvector() {
    SqlSearchQueryCondition result = build(SqlClause.create(Operand.AND,
        new SqlPredicate(Operand.EQ, SearchField.FORMAT, new ExactTerm("test")),
        new SqlPredicate(Operand.EQ, SearchField.NAMESPACE, new ExactTerm("org.test")),
        new SqlPredicate(Operand.EQ, SearchField.NAME, new ExactTerm("artifact"))));

    assertThat(result.getSqlFilter(), is("(cs.format = #{filterParams.cs_format0} AND "
        + "cs.namespace = #{filterParams.cs_namespace1} AND "
        + "cs.search_component_name = #{filterParams.cs_search_component_name2})"));
  }

  @Test
  void testInOperatorOnPlainColumnIsExact() {
    SqlSearchQueryCondition result = build(new SqlPredicate(Operand.IN, SearchField.REPOSITORY_NAME,
        TermCollection.create(new ExactTerm("maven-releases"), new ExactTerm("maven"))));

    assertThat(result.getSqlFilter(), is("cs.search_repository_name IN (#{filterParams.cs_search_repository_name0}, "
        + "#{filterParams.cs_search_repository_name1})"));
    assertThat(result.getParameters(),
        is(Map.of("cs_search_repository_name0", "maven-releases", "cs_search_repository_name1", "maven")));
  }

  @Test
  void testWildcardOnTextSearchableColumnIsCaseInsensitivePrefix() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.NAME, new WildcardTerm("Spring-Boot*")));

    assertThat(result.getSqlFilter(),
        is("(cs.tsvector_search_component_name @@ #{filterParams.cs_tsvector_search_component_name1}::tsquery AND "
            + "LOWER(cs.search_component_name) LIKE LOWER(#{filterParams.cs_search_component_name0}))"));
    assertThat(result.getParameters(), is(Map.of("cs_search_component_name0", "Spring-Boot%",
        "cs_tsvector_search_component_name1", "'spring-boot':*")));
  }

  @Test
  void testWildcardPrefixEscapesLikeCharacters() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.VERSION, new WildcardTerm("1_0%\\")));

    assertThat(result.getParameters().get("cs_version0"), is("1\\_0\\%\\\\%"));
    assertThat(result.getParameters().get("cs_tsvector_version1"), is("'1_0%\\\\':*"));
  }

  @Test
  void testWildcardOnPlainOnlyColumnUsesLikePrefix() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.REPOSITORY_NAME, new WildcardTerm("maven-*")));

    assertThat(result.getSqlFilter(),
        is("LOWER(cs.search_repository_name) LIKE LOWER(#{filterParams.cs_search_repository_name0})"));
    assertThat(result.getParameters().get("cs_search_repository_name0"), is("maven-%"));
  }

  @Test
  void testMultipleTermsOnTextSearchableColumnUseTsvector() {
    SqlSearchQueryCondition result = build(new SqlPredicate(Operand.EQ, SearchField.NAME,
        TermCollection.create(new WildcardTerm("maven"), new LenientTerm("provider"))));

    assertThat(result.getSqlFilter(),
        is("cs.tsvector_search_component_name @@ #{filterParams.cs_tsvector_search_component_name0}::tsquery"));
    assertThat(result.getParameters().get("cs_tsvector_search_component_name0"), is("('maven':*) & ('provider')"));
  }

  @Test
  void testLenientTermOnPlainOnlyColumnUsesContains() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.COMPONENT_KIND, new LenientTerm("jar")));

    assertThat(result.getSqlFilter(), is("(LOWER(cs.component_kind) LIKE LOWER(#{filterParams.cs_component_kind0}))"));
    assertThat(result.getParameters().get("cs_component_kind0"), is("%jar%"));
  }

  @Test
  void testRegexWildcardTermOnName() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.REGEX, SearchField.NAME, new RegexWildcardTerm("goo*com*")));

    assertThat(result.getSqlFilter(), is("cs.search_component_name ~ #{filterParams.cs_search_component_name0}"));
    assertThat(result.getParameters().get("cs_search_component_name0"), is("^goo.*com.*$"));
  }

  @Test
  void testRegexOnPathsUsesVarcharColumn() {
    SqlSearchQueryCondition result = build(
        new SqlPredicate(Operand.REGEX, SearchField.PATHS, new ExactTerm("(^|{')(/org/.*)('}|$)")));

    assertThat(result.getSqlFilter(), is("cs.paths ~ #{filterParams.cs_paths0}"));
    assertThat(result.getParameters().get("cs_paths0"), is("(^|{')(/org/.*)('}|$)"));
  }

  @Test
  void testRegexOnTsvectorOnlyColumnIsRejected() {
    assertThrows(ValidationErrorsException.class,
        () -> build(new SqlPredicate(Operand.REGEX, SearchField.KEYWORDS, new ExactTerm(".*"))));
  }

  @Test
  void testPathEqualityUsesTsvector() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.PATHS, new ExactTerm("/Org/Foo/Bar-1.0.jar")));

    assertThat(result.getSqlFilter(), is("cs.tsvector_paths @@ #{filterParams.cs_tsvector_paths0}::tsquery"));
    assertThat(result.getParameters().get("cs_tsvector_paths0"), is("'/org/foo/bar-1.0.jar'"));
  }

  @Test
  void testContentSelectorPathPrefixIsNotTokenized() {
    SqlSearchQueryCondition result = build(
        new SqlPredicate(Operand.EQ, SearchField.PATHS, new WildcardTerm("/v2/pws-blueprint/", false)));

    assertThat(result.getSqlFilter(), is("cs.tsvector_paths @@ #{filterParams.cs_tsvector_paths0}::tsquery"));
    assertThat(result.getParameters().get("cs_tsvector_paths0"), is("'/v2/pws-blueprint/':*"));
  }

  @Test
  void testNotEqualOnTsvectorColumnNegatesMatch() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.NOT_EQ, SearchField.PATHS, new ExactTerm("/foo")));

    assertThat(result.getSqlFilter(), is("NOT (cs.tsvector_paths @@ #{filterParams.cs_tsvector_paths0}::tsquery)"));
    assertThat(result.getParameters().get("cs_tsvector_paths0"), is("'/foo'"));
  }

  @Test
  void testNotEqualOnPlainColumn() {
    SqlSearchQueryCondition result = build(new SqlPredicate(Operand.NOT_EQ, SearchField.FORMAT, new ExactTerm("npm")));

    assertThat(result.getSqlFilter(), is("cs.format <> #{filterParams.cs_format0}"));
  }

  @Test
  void testNullChecks() {
    assertThat(build(new SqlPredicate(Operand.EQ, SearchField.PATHS, NullTerm.INSTANCE)).getSqlFilter(),
        is("cs.tsvector_paths IS NULL"));
    assertThat(build(new SqlPredicate(Operand.NOT_EQ, SearchField.NAMESPACE, NullTerm.INSTANCE)).getSqlFilter(),
        is("cs.namespace IS NOT NULL"));
  }

  @Test
  void testTagsAreMatchedCaseInsensitively() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.TAGS, new ExactTerm("my-Tag-With-Mixed-Case")));

    assertThat(result.getSqlFilter(), is("cs.tsvector_tags @@ #{filterParams.cs_tsvector_tags0}::tsquery"));
    assertThat(result.getParameters().get("cs_tsvector_tags0"), is("'my-tag-with-mixed-case'"));
  }

  @Test
  void testTimestampColumnIsCast() {
    SqlSearchQueryCondition result =
        build(new SqlPredicate(Operand.EQ, SearchField.LAST_MODIFIED, new ExactTerm("2024-01-01T00:00:00Z")));

    assertThat(result.getSqlFilter(),
        is("cs.last_modified = CAST(#{filterParams.cs_last_modified0} AS TIMESTAMP WITH TIME ZONE)"));
  }

  @Test
  void testWildcardOnTimestampColumnIsRejected() {
    assertThrows(ValidationErrorsException.class,
        () -> build(new SqlPredicate(Operand.EQ, SearchField.LAST_MODIFIED, new WildcardTerm("2024"))));
  }

  @Test
  void testTooLongTermIsRejected() {
    assertThrows(ValidationErrorsException.class,
        () -> build(new SqlPredicate(Operand.EQ, SearchField.KEYWORDS, new ExactTerm("a".repeat(3000)))));
  }

  @Test
  void testClauseWithAllEmptyChildrenProducesEmptyFilter() {
    SqlSearchQueryCondition result = build(SqlClause.create(Operand.OR,
        new SqlPredicate(Operand.EQ, SearchField.KEYWORDS, new ExactTerm("")),
        new SqlPredicate(Operand.EQ, SearchField.KEYWORDS, new WildcardTerm("  ")),
        new SqlPredicate(Operand.EQ, SearchField.KEYWORDS, new LenientTerm(" "))));

    assertThat(result.getSqlFilter(), is(""));
    assertThat(result.getParameters().isEmpty(), is(true));
  }

  @Test
  void testNestedClauseWithCollapsingInnerClauseKeepsOnlySurvivingSibling() {
    Expression innerAllEmpty = SqlClause.create(Operand.OR,
        new SqlPredicate(Operand.EQ, SearchField.KEYWORDS, new ExactTerm("")),
        new SqlPredicate(Operand.EQ, SearchField.KEYWORDS, new WildcardTerm("  ")));
    SqlSearchQueryCondition result = build(SqlClause.create(Operand.AND,
        new SqlPredicate(Operand.EQ, SearchField.KEYWORDS, new WildcardTerm("lodash")), innerAllEmpty));

    assertThat(result.getSqlFilter(), is("(cs.keywords @@ #{filterParams.cs_keywords0}::tsquery)"));
    assertThat(result.getParameters().get("cs_keywords0"), is("'lodash':*"));
  }

  @Test
  void testMixedTermTypesInClause() {
    SqlSearchQueryCondition result = build(SqlClause.create(Operand.OR,
        new SqlPredicate(Operand.EQ, SearchField.KEYWORDS, new ExactTerm("java")),
        new SqlPredicate(Operand.EQ, SearchField.PRERELEASE, new BooleanTerm(false))));

    assertThat(result.getSqlFilter(), is("(cs.keywords @@ #{filterParams.cs_keywords0}::tsquery OR "
        + "cs.prerelease = #{filterParams.cs_prerelease1})"));
  }

  @Test
  void testFactoryUsesAssetParametersForAssetFilters() {
    SqlSearchQueryConditionGroup group = new PostgresSearchConditionFactory(new PostgresSearchDB()).build(
        new ExpressionGroup(new SqlPredicate(Operand.EQ, SearchField.NAME, new ExactTerm("a")),
            new SqlPredicate(Operand.EQ, SearchField.ASSET_FORMAT_VALUE_1, new WildcardTerm("x86"))));

    assertThat(group.getComponentCondition().getSqlFilter(),
        is("cs.search_component_name = #{filterParams.cs_search_component_name0}"));
    SqlSearchQueryCondition assetCondition = group.getAssetCondition().orElseThrow();
    assertThat(assetCondition.getSqlFilter(),
        is("LOWER(ap.asset_format_value_1) LIKE LOWER(#{assetFilterParams.ap_asset_format_value_10})"));
    assertThat(assetCondition.getParameters().get("ap_asset_format_value_10"), is("x86%"));
  }

  private static SqlSearchQueryCondition build(final Expression expression) {
    return new PostgresFulltextSearchConditionBuilder(new PostgresSearchDB(), ConditionType.COMPONENT_FILTER)
        .build(expression);
  }
}
