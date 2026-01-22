
package org.globalart.astrum.blobstore.api;

import java.util.Collections;
import java.util.List;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.Matchers.sameInstance;

/**
 * Tests for {@link PaginatedResult}.
 */
public class PaginatedResultTest
{
  private static final String TOKEN = "next-continuation-token";

  @Test
  public void testGetResults() {
    List<String> results = List.of("a", "b", "c");

    PaginatedResult<String> underTest = new PaginatedResult<>(results, TOKEN);

    assertThat(underTest.getResults(), is(sameInstance(results)));
    assertThat(underTest.getResults(), contains("a", "b", "c"));
  }

  @Test
  public void testGetNextContinuationToken() {
    PaginatedResult<String> underTest = new PaginatedResult<>(List.of("a"), TOKEN);

    assertThat(underTest.getNextContinuationToken(), is(TOKEN));
  }

  @Test
  public void testEmptyResults() {
    PaginatedResult<String> underTest = new PaginatedResult<>(Collections.emptyList(), null);

    assertThat(underTest.getResults(), is(empty()));
    assertThat(underTest.getNextContinuationToken(), is(nullValue()));
  }

  @Test
  public void testNullResultsAndToken() {
    PaginatedResult<String> underTest = new PaginatedResult<>(null, null);

    assertThat(underTest.getResults(), is(nullValue()));
    assertThat(underTest.getNextContinuationToken(), is(nullValue()));
  }

  @Test
  public void testConstructorStoresArgumentsInCorrectFields() {
    List<String> results = List.of("x", "y");

    PaginatedResult<String> underTest = new PaginatedResult<>(results, TOKEN);

    assertThat(underTest.getResults(), is(sameInstance(results)));
    assertThat(underTest.getNextContinuationToken(), is(sameInstance(TOKEN)));
  }
}
