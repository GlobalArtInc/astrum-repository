
package org.globalart.astrum.content.maven.internal.search.table;

import java.util.Arrays;
import java.util.Optional;

import org.globalart.astrum.repository.maven.internal.search.MavenSearchMappings;
import org.globalart.astrum.repository.rest.sql.SearchField;
import org.globalart.astrum.repository.search.query.SearchFilter;
import org.globalart.astrum.repository.search.sql.SearchMappingService;
import org.globalart.astrum.repository.search.sql.query.syntax.ExactTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.Expression;
import org.globalart.astrum.repository.search.sql.query.syntax.Operand;
import org.globalart.astrum.repository.search.sql.query.syntax.SqlPredicate;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertTrue;
import static org.globalart.astrum.content.maven.internal.search.table.MavenBaseVersionSqlSearchQueryContribution.BASE_VERSION;

public class MavenBaseVersionSqlSearchQueryContributionTest

{
  private MavenBaseVersionSqlSearchQueryContribution underTest;

  @Before
  public void setup() {
    underTest = new MavenBaseVersionSqlSearchQueryContribution();
    underTest.init(new SearchMappingService(Arrays.asList(new MavenSearchMappings())), false);
  }

  @Test
  public void shouldContributeCustomisedBaseVersion() {
    String baseVersion = "1.0-snapshot";

    Optional<Expression> result = underTest.createPredicate(new SearchFilter(BASE_VERSION, baseVersion));

    assertTrue(result.isPresent());

    assertThat(result.get(), is(new SqlPredicate(Operand.EQ, SearchField.FORMAT_FIELD_1, new ExactTerm(baseVersion))));
  }
}
