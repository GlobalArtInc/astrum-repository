
package org.globalart.astrum.cleanup.internal.datastore.search.criteria;

import java.util.function.Predicate;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class RegexCleanupEvaluatorTest

{

  @Mock
  private Repository repository;

  private RegexCleanupEvaluator underTest;

  @BeforeEach
  void setup() {
    underTest = new RegexCleanupEvaluator();
  }

  @ParameterizedTest
  @CsvSource({
      ".*\\.jar, test.jar, true",
      ".*\\.jar, test.txt, false",
      "^src/.*, src/main.java, true",
      "^src/.*, bin/main.class, false"
  })
  void testValidRegexPatterns(final String regex, final String assetPath, final boolean expectedResult) {
    Asset asset = mock(Asset.class);
    when(asset.path()).thenReturn(assetPath);

    Predicate<Asset> predicate = underTest.getPredicate(repository, regex);
    assertThat(predicate.test(asset), is(expectedResult));
  }

  @Test
  void testInvalidRegexThrowsException() {
    String invalidRegex = "[a-z";// Invalid regex pattern
    when(repository.getName()).thenReturn("test-repo");
    assertThrows(RuntimeException.class, () -> underTest.getPredicate(repository, invalidRegex));
  }
}
