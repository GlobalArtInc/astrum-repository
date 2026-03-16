
package org.globalart.astrum.repository.search.sql.query.h2;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.nullValue;

class H2SearchDBTest
{
  private H2SearchDB underTest;

  @BeforeEach
  void setup() {
    underTest = new H2SearchDB();
  }

  @Test
  void shouldNotFormatJsonPathColumn() {
    String result = underTest.formatJsonPath("cs.attributes", "nuget.id");
    assertThat(result, nullValue());
  }
}
